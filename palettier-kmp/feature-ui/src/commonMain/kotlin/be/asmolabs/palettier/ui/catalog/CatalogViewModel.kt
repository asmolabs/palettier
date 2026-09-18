package be.asmolabs.palettier.ui.catalog

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.color.deltaE2000
import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.port.PaintCatalogRepository
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch

/**
 * Le catalogue : ce qui existe, et ce qui s'approche d'une teinte.
 *
 * <p>Le filtrage se fait en memoire sur la liste rendue par le depot, et non par une
 * requete par frappe. Sept cents tubes tiennent largement en memoire, et le flux de la
 * base reste ainsi la seule source : declarer un tube possede met la liste a jour sans
 * qu'on la redemande.</p>
 */
class CatalogViewModel(
    private val catalog: PaintCatalogRepository,
) : ViewModel() {

    private val query = MutableStateFlow("")
    private val onlyInStock = MutableStateFlow(false)
    private val target = MutableStateFlow<String?>(null)

    val state: StateFlow<CatalogUiState> =
        combine(catalog.observeAll(), query, onlyInStock, target) { paints, text, inStock, hex ->
            val kept = paints
                .filter { !inStock || it.inStock }
                .filter { matches(it, text) }

            CatalogUiState(
                loading = false,
                rows = rank(kept, hex),
                total = paints.size,
                query = text,
                onlyInStock = inStock,
                targetHex = hex,
            )
        }
            .catch { emit(CatalogUiState(loading = false, error = it.message ?: "Lecture impossible")) }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5_000), CatalogUiState())

    fun onIntent(intent: CatalogIntent) {
        when (intent) {
            is CatalogIntent.Search -> query.value = intent.query
            is CatalogIntent.OnlyInStock -> onlyInStock.value = intent.only
            is CatalogIntent.AimAt -> target.value = intent.hex?.takeIf { it.isNotBlank() }
            is CatalogIntent.SetOwned -> viewModelScope.launch {
                catalog.setOwned(intent.paint, intent.owned)
            }
        }
    }

    /**
     * La recherche porte sur le nom, la marque, la reference -- ancienne comprise -- et
     * les pigments.
     *
     * <p>L'ancienne reference compte : un tube achete avant une renumerotation porte
     * encore son etiquette d'origine, et c'est sous celle-la que le peintre le cherche.</p>
     */
    private fun matches(paint: Paint, text: String): Boolean {
        if (text.isBlank()) return true
        val needle = text.trim().lowercase()
        return paint.name.lowercase().contains(needle) ||
            paint.brand.lowercase().contains(needle) ||
            paint.code.lowercase().contains(needle) ||
            paint.legacyCode.lowercase().contains(needle) ||
            paint.pigments.any { it.lowercase().contains(needle) }
    }

    /**
     * Par ecart quand une teinte est visee, par marque et nom sinon.
     *
     * <p>Les deux classements repondent a deux questions differentes : "ou est ce tube"
     * et "avec quoi puis-je faire cette couleur".</p>
     */
    private fun rank(paints: List<Paint>, hex: String?): List<CatalogRow> {
        val aim = hex?.let { runCatching { Rgb.ofHex(it) }.getOrNull() }
            ?: return paints
                .sortedWith(compareBy({ it.brand }, { it.name }))
                .map { CatalogRow(it, null) }

        return paints
            .map { CatalogRow(it, deltaE2000(aim, it.color)) }
            .sortedBy { it.deltaE }
    }
}
