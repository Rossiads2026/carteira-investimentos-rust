use askama::Template;

pub struct AssetDisplay {
    pub symbol: String,
    pub asset_type: String,
    pub quantity: String,
    pub average_price: String,
    pub total_invested: String,
}

#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
    pub email: String,
    pub assets: Vec<AssetDisplay>,
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub error: Option<String>,
}
