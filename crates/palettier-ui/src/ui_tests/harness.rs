//! A small driver over a headless `VirtualDom`.
//!
//! The tree is read straight from the mounted templates, so every listener comes with
//! the `ElementId` the runtime dispatches on: no HTML parsing, no hydration counters.

use crate::state::{AppServices, AppState};
use crate::{StudioShell, StudioShellProps};
use dioxus::dioxus_core::{
    AttributeValue, DynamicNode, ElementId, Event, NoOpMutations, TemplateAttribute, TemplateNode,
    VNode, VirtualDom,
};
use dioxus::html::{
    PlatformEventData, SerializedFormData, SerializedHtmlEventConverter, SerializedKeyboardData,
    SerializedMouseData,
};
use dioxus::prelude::{Code, Key, Location, Modifiers};
use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Once;
use std::time::{Duration, Instant};

/// How long the app may stay quiet before the screen counts as settled. Database calls
/// wake the scheduler when they finish, so this only has to outlast one of them.
const IDLE: Duration = Duration::from_millis(150);
/// Upper bound on settling, for screens that keep a ticking timer.
const SETTLE_LIMIT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone)]
pub enum Node {
    Element(Element),
    Text(String),
}

#[derive(Debug, Clone)]
pub struct Element {
    pub tag: String,
    pub attrs: Vec<(String, String)>,
    /// Event name without the `on` prefix, and the element it is mounted on.
    pub listeners: Vec<(String, ElementId)>,
    pub children: Vec<Node>,
}

impl Element {
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .rev()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }

    pub fn has_class(&self, class: &str) -> bool {
        self.attrs
            .iter()
            .filter(|(n, _)| n == "class")
            .any(|(_, v)| v.split_whitespace().any(|c| c == class))
    }

    pub fn listener(&self, event: &str) -> Option<ElementId> {
        self.listeners
            .iter()
            .find(|(n, _)| n == event)
            .map(|(_, id)| *id)
    }

    /// Visible text, whitespace collapsed.
    pub fn text(&self) -> String {
        let mut out = String::new();
        collect_text(&self.children, &mut out);
        squash(&out)
    }

    /// Every element below this one (itself included), in document order.
    pub fn descendants(&self) -> Vec<&Element> {
        let mut out = vec![self];
        for child in &self.children {
            if let Node::Element(el) = child {
                out.extend(el.descendants());
            }
        }
        out
    }
}

fn collect_text(nodes: &[Node], out: &mut String) {
    for node in nodes {
        match node {
            Node::Text(t) => {
                out.push_str(t);
                out.push(' ');
            }
            Node::Element(el) => {
                // Options of a closed <select> are not on screen.
                if el.tag != "option" {
                    collect_text(&el.children, out);
                }
            }
        }
    }
}

fn squash(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub struct Ui {
    dom: VirtualDom,
    pub services: AppServices,
}

impl Ui {
    /// The full studio shell on a fresh in-memory database, data loaded.
    pub async fn studio() -> Self {
        static CONVERTER: Once = Once::new();
        CONVERTER.call_once(|| {
            dioxus::html::set_event_converter(Box::new(SerializedHtmlEventConverter));
        });
        let services = AppServices::in_memory().await.expect("in-memory services");
        let mut dom = VirtualDom::new_with_props(
            StudioShell,
            StudioShellProps {
                services: services.clone(),
            },
        );
        dom.rebuild_in_place();
        let mut ui = Ui { dom, services };
        ui.settle().await;
        ui
    }

    /// Run tasks and re-render until nothing is left to do.
    pub async fn settle(&mut self) {
        let start = Instant::now();
        while start.elapsed() < SETTLE_LIMIT {
            match tokio::time::timeout(IDLE, self.dom.wait_for_work()).await {
                Ok(()) => self.dom.render_immediate(&mut NoOpMutations),
                Err(_) => break,
            }
        }
        self.dom.render_immediate(&mut NoOpMutations);
    }

    /// The shared app state, as the views see it.
    pub fn state(&self) -> AppState {
        self.dom
            .in_runtime(|| dioxus::dioxus_core::ScopeId::APP.consume_context::<AppState>())
            .expect("AppState is provided by StudioShell")
    }

    /// Read the state inside the runtime (signals are not readable outside it).
    pub fn peek<T>(&self, f: impl FnOnce(&AppState) -> T) -> T {
        let state = self.state();
        self.dom.in_runtime(|| f(&state))
    }

    pub fn tree(&self) -> Element {
        let mut roots = Vec::new();
        walk_vnode(&self.dom, self.dom.base_scope().root_node(), &mut roots);
        Element {
            tag: "#root".into(),
            attrs: Vec::new(),
            listeners: Vec::new(),
            children: roots,
        }
    }

    /// All visible text on screen.
    pub fn text(&self) -> String {
        self.tree().text()
    }

    /// Wait until `ready` holds, rendering whatever work comes in meanwhile. Gives up
    /// after [`SETTLE_LIMIT`]: a loaded machine can be slower than [`IDLE`].
    async fn wait_until<T>(&mut self, ready: impl Fn(&Self) -> Option<T>) -> Option<T> {
        let start = Instant::now();
        loop {
            if let Some(found) = ready(self) {
                return Some(found);
            }
            if start.elapsed() > SETTLE_LIMIT {
                return None;
            }
            if tokio::time::timeout(IDLE, self.dom.wait_for_work())
                .await
                .is_ok()
            {
                self.dom.render_immediate(&mut NoOpMutations);
            }
        }
    }

    pub async fn assert_shows(&mut self, needle: &str) {
        let shown = self
            .wait_until(|ui| ui.text().contains(needle).then_some(()))
            .await;
        assert!(
            shown.is_some(),
            "expected the screen to show {needle:?}; it shows:\n{}",
            self.text()
        );
    }

    pub async fn assert_hides(&mut self, needle: &str) {
        let hidden = self
            .wait_until(|ui| (!ui.text().contains(needle)).then_some(()))
            .await;
        assert!(
            hidden.is_some(),
            "expected the screen not to show {needle:?}; it shows:\n{}",
            self.text()
        );
    }

    /// The first element with this class, if any.
    pub fn find_class(&self, class: &str) -> Option<Element> {
        self.tree()
            .descendants()
            .into_iter()
            .find(|e| e.has_class(class))
            .cloned()
    }

    /// Click the clickable element whose text (or title) is `label`. An exact match wins
    /// over one that only contains it; among those, the innermost one.
    pub async fn click(&mut self, label: &str) {
        let id = self.wait_clickable(label, None).await;
        self.fire_mouse(id).await;
    }

    /// Same as [`Ui::click`], restricted to the element with class `scope`.
    pub async fn click_in(&mut self, scope: &str, label: &str) {
        let id = self.wait_clickable(label, Some(scope)).await;
        self.fire_mouse(id).await;
    }

    async fn wait_clickable(&mut self, label: &str, scope: Option<&str>) -> ElementId {
        if let Some(id) = self.wait_until(|ui| ui.clickable(label, scope).ok()).await {
            return id;
        }
        match self.clickable(label, scope) {
            Ok(id) => id,
            Err(why) => panic!("{why}"),
        }
    }

    fn clickable(&self, label: &str, scope: Option<&str>) -> Result<ElementId, String> {
        let tree = self.tree();
        let root = match scope {
            Some(class) => match tree.descendants().into_iter().find(|e| e.has_class(class)) {
                Some(el) => el.clone(),
                None => return Err(format!("no element with class {class:?}")),
            },
            None => tree,
        };
        let candidates: Vec<&Element> = root
            .descendants()
            .into_iter()
            .filter(|e| e.listener("click").is_some())
            .collect();
        let exact = |e: &Element| e.text() == label || e.attr("title") == Some(label);
        let loose = |e: &Element| e.text().contains(label);
        // Innermost match: no clickable element inside it matches as well.
        let innermost = |test: &dyn Fn(&Element) -> bool| {
            candidates.iter().copied().find(|e| {
                test(e)
                    && !e.descendants()[1..]
                        .iter()
                        .any(|c| c.listener("click").is_some() && test(c))
            })
        };
        let pick = innermost(&exact).or_else(|| innermost(&loose));
        match pick {
            Some(el) => Ok(el.listener("click").expect("filtered on click")),
            None => {
                let labels: Vec<String> = candidates
                    .iter()
                    .map(|e| e.text())
                    .filter(|t| !t.is_empty())
                    .collect();
                Err(format!(
                    "nothing clickable labelled {label:?}; clickable: {labels:#?}"
                ))
            }
        }
    }

    /// Type into the field whose placeholder or title is `label` (exact, then contains).
    pub async fn type_into(&mut self, label: &str, value: &str) {
        let id = match self.wait_until(|ui| ui.field(label).ok()).await {
            Some(id) => id,
            None => match self.field(label) {
                Ok(id) => id,
                Err(why) => panic!("{why}"),
            },
        };
        self.fire_form(id, value).await;
    }

    fn field(&self, label: &str) -> Result<ElementId, String> {
        let tree = self.tree();
        let fields: Vec<&Element> = tree
            .descendants()
            .into_iter()
            .filter(|e| e.listener("input").is_some() || e.listener("change").is_some())
            .collect();
        let matches = |e: &Element, exact: bool| {
            ["placeholder", "title", "aria-label"].iter().any(|a| {
                e.attr(a)
                    .is_some_and(|v| if exact { v == label } else { v.contains(label) })
            })
        };
        let field = fields
            .iter()
            .find(|e| matches(e, true))
            .or_else(|| fields.iter().find(|e| matches(e, false)));
        match field {
            Some(f) => Ok(f
                .listener("input")
                .or(f.listener("change"))
                .expect("filtered on input/change")),
            None => {
                let names: Vec<&str> = fields
                    .iter()
                    .filter_map(|e| e.attr("placeholder").or(e.attr("title")))
                    .collect();
                Err(format!("no field labelled {label:?}; fields: {names:#?}"))
            }
        }
    }

    /// Press ⌘ + `ch` on the app shell, the way the global shortcuts are read.
    pub async fn press_cmd(&mut self, ch: &str) {
        let shell = self
            .tree()
            .descendants()
            .into_iter()
            .find(|e| e.has_class("app-shell"))
            .and_then(|e| e.listener("keydown"))
            .expect("the app shell listens to keydown");
        let data = SerializedKeyboardData::new(
            Key::Character(ch.to_string()),
            Code::Unidentified,
            Location::Standard,
            false,
            Modifiers::META,
            false,
        );
        self.fire("keydown", shell, Box::new(data), true);
        self.settle().await;
    }

    async fn fire_mouse(&mut self, id: ElementId) {
        self.fire("click", id, Box::new(SerializedMouseData::default()), true);
        self.settle().await;
    }

    async fn fire_form(&mut self, id: ElementId, value: &str) {
        // The desktop renderer sends both, like a browser does for text fields.
        for name in ["input", "change"] {
            let data = SerializedFormData::new(value.to_string(), HashMap::new());
            self.fire(name, id, Box::new(data), true);
        }
        self.settle().await;
    }

    fn fire(&mut self, name: &str, id: ElementId, data: Box<dyn Any>, bubbles: bool) {
        let event = Event::new(
            Rc::new(PlatformEventData::new(data)) as Rc<dyn Any>,
            bubbles,
        );
        self.dom.runtime().handle_event(name, event, id);
    }
}

fn walk_vnode(dom: &VirtualDom, vnode: &VNode, out: &mut Vec<Node>) {
    for root in vnode.template.roots.iter() {
        walk_template(dom, vnode, root, out);
    }
}

fn walk_template(dom: &VirtualDom, vnode: &VNode, node: &TemplateNode, out: &mut Vec<Node>) {
    match node {
        TemplateNode::Element {
            tag,
            attrs,
            children,
            ..
        } => {
            let mut el = Element {
                tag: tag.to_string(),
                attrs: Vec::new(),
                listeners: Vec::new(),
                children: Vec::new(),
            };
            for attr in attrs.iter() {
                match attr {
                    TemplateAttribute::Static { name, value, .. } => {
                        el.attrs.push((name.to_string(), value.to_string()));
                    }
                    TemplateAttribute::Dynamic { id } => {
                        let mounted = vnode.mounted_dynamic_attribute(*id, dom);
                        for a in vnode.dynamic_attrs[*id].iter() {
                            let value = match &a.value {
                                AttributeValue::Text(s) => s.clone(),
                                AttributeValue::Int(i) => i.to_string(),
                                AttributeValue::Float(f) => f.to_string(),
                                AttributeValue::Bool(b) => b.to_string(),
                                AttributeValue::Listener(_) => {
                                    if let Some(id) = mounted {
                                        let event = a.name.trim_start_matches("on");
                                        el.listeners.push((event.to_string(), id));
                                    }
                                    continue;
                                }
                                AttributeValue::Any(_) | AttributeValue::None => continue,
                            };
                            el.attrs.push((a.name.to_string(), value));
                        }
                    }
                }
            }
            for child in children.iter() {
                walk_template(dom, vnode, child, &mut el.children);
            }
            out.push(Node::Element(el));
        }
        TemplateNode::Text { text } => out.push(Node::Text(text.to_string())),
        TemplateNode::Dynamic { id } => match &vnode.dynamic_nodes[*id] {
            DynamicNode::Component(component) => {
                if let Some(scope) = component.mounted_scope(*id, vnode, dom) {
                    walk_vnode(dom, scope.root_node(), out);
                }
            }
            DynamicNode::Text(text) => out.push(Node::Text(text.value.clone())),
            DynamicNode::Placeholder(_) => {}
            DynamicNode::Fragment(nodes) => {
                for child in nodes {
                    walk_vnode(dom, child, out);
                }
            }
        },
    }
}
