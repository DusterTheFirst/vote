use askama::Template;

#[derive(Template, Debug)]
#[template(path = "form.html")]
pub struct FormTemplate {
    pub username: String,
    pub discriminator: String,
    pub token: String,
    pub email: String,
    pub form_embed_url: String,
}
