pub struct UiPage {
    components: Vec<Box<dyn UiPageComponent>>
}

trait UiPageComponent {}

#[derive(Debug)]
pub enum MainPageContent {
    QueryInput,
    HostsTable,
    ProgresGauge,
}

impl UiPageComponent for MainPageContent {}

// Make a macro
// impl UiComponent {
//     pub fn iterator() -> Iter<'static, UiComponent> {
//         static DIRECTIONS: [UiComponent; 4] = [QueryInput, HostsTable, ProgresGauge]
//         DIRECTIONS.iter()
//     }
// }
