package be.asmolabs.palettier.ui.app

import be.asmolabs.palettier.domain.plan.ProjectPlanner
import be.asmolabs.palettier.domain.plan.SampledPlanService
import be.asmolabs.palettier.domain.recipe.RecipeTimelineService
import be.asmolabs.palettier.image.imageDecoder
import be.asmolabs.palettier.ui.SampledColour
import be.asmolabs.palettier.ui.assistant.AssistantViewModel
import be.asmolabs.palettier.ui.catalog.CatalogViewModel
import be.asmolabs.palettier.ui.drying.DryingViewModel
import be.asmolabs.palettier.ui.mixer.MixerViewModel
import be.asmolabs.palettier.ui.palettes.PalettesViewModel
import be.asmolabs.palettier.ui.picker.PickerViewModel
import be.asmolabs.palettier.ui.projects.ProjectsViewModel
import be.asmolabs.palettier.ui.recipes.RecipesViewModel
import be.asmolabs.palettier.ui.workbench.WorkbenchViewModel
import org.koin.dsl.module

/**
 * Les ViewModels et ce qu'ils partagent.
 *
 * <p>Declares ici plutot que dans chaque client : ce sont les memes ecrans des deux
 * cotes, et les brancher deux fois ferait diverger les deux applications au premier
 * parametre ajoute. Ce qui differe d'une plateforme a l'autre -- le pilote de base, le
 * choix d'un fichier -- reste chez elle.</p>
 */
val uiModule = module {
    // Un seul relais pour toute l'application : la pipette y depose sa mesure, le
    // catalogue, le melangeur et l'etabli la retrouvent sans se connaitre.
    single { SampledColour() }

    single { RecipeTimelineService(get()) }
    single { ProjectPlanner(get()) }
    single { SampledPlanService(get()) }

    factory { WorkbenchViewModel(get(), get(), get(), get(), get()) }
    factory { ProjectsViewModel(get(), get(), get(), get(), get(), imageDecoder(), get()) }
    factory { CatalogViewModel(get(), get()) }
    factory { PickerViewModel(imageDecoder(), get(), get(), get(), get(), get(), get()) }
    factory { PalettesViewModel(get(), get()) }
    factory { MixerViewModel(get(), get(), get(), get()) }
    factory { DryingViewModel(get()) }
    factory { RecipesViewModel(get(), get()) }
    factory { AssistantViewModel(get(), get(), get(), get(), get(), get(), get(), imageDecoder()) }
}
