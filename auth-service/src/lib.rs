use axum::{
    Router,
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use axum_extra::extract::CookieJar;

pub trait AuthState: Clone + Send + Sync + 'static {
    fn on_authenticated(
        &self,
        jar: CookieJar,
        headers: &HeaderMap,
    ) -> impl std::future::Future<Output = Response> + Send;

    fn logout_session(
        &self,
        jar: CookieJar,
    ) -> impl std::future::Future<Output = Option<CookieJar>> + Send;
}

pub async fn handle_logout<S: AuthState>(State(state): State<S>, jar: CookieJar) -> Response {
    let Some(cleared_jar) = state.logout_session(jar).await else {
        return Redirect::to("/").into_response();
    };

    let mut response = Redirect::to("/").into_response();
    response.extensions_mut().insert(cleared_jar);

    response
}

pub async fn handle_login<S: AuthState>(
    State(state): State<S>,
    jar: CookieJar,
    headers: HeaderMap,
) -> Response {
    state.on_authenticated(jar, &headers).await
}

pub fn router<S: AuthState>() -> Router<S> {
    Router::new()
        .route("/login", get(handle_login::<S>))
        .route("/logout", get(handle_logout::<S>))
}
