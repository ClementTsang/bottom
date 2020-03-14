use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename = "row")]
pub struct Row {
    ratio: Option<u64>,
    percent: Option<u64>,
    child: Option<Vec<RowChildren>>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum RowChildren {
    Widget(FinalWidget),
    Col {
        ratio: Option<u64>,
        percent: Option<u64>,
        child: Vec<FinalWidget>,
    },
}

#[derive(Deserialize, Debug)]
pub struct FinalWidget {
    ratio: Option<u64>,
    percent: Option<u64>,
    #[serde(rename = "type")]
    widget: String,
}
