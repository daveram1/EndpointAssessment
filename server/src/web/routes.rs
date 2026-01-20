use axum::{
    extract::{Path, State, Form},
    response::{Html, IntoResponse, Redirect, Response},
};
use axum_extra::extract::cookie::CookieJar;
use common::{AdminRole, CheckStatus, Severity};
use serde::Deserialize;
use uuid::Uuid;

use crate::AppState;
use crate::db::{checks, endpoints, results, snapshots, users};
use crate::web::auth::{
    create_session_cookie, clear_session_cookie, hash_password, verify_password,
    AuthenticatedUser, Session,
};
use crate::web::templates::*;

// Dashboard
pub async fn dashboard(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> impl IntoResponse {
    let endpoint_counts = endpoints::get_endpoint_counts(&state.pool)
        .await
        .unwrap_or(crate::db::endpoints::EndpointCounts {
            total: 0,
            online: 0,
            offline: 0,
            warning: 0,
            critical: 0,
        });

    let check_counts = checks::get_check_counts(&state.pool)
        .await
        .unwrap_or(crate::db::checks::CheckCounts { total: 0, enabled: 0 });

    let recent = results::get_recent_results(&state.pool, 10)
        .await
        .unwrap_or_default();

    let recent_results: Vec<RecentResultView> = recent
        .into_iter()
        .map(|r| RecentResultView {
            endpoint_hostname: r.endpoint_hostname,
            check_name: r.check_name,
            status: r.status.parse().unwrap_or(CheckStatus::Error),
            message: r.message,
            collected_at: r.collected_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        })
        .collect();

    DashboardTemplate {
        title: "Dashboard".to_string(),
        total_endpoints: endpoint_counts.total,
        online_endpoints: endpoint_counts.online,
        offline_endpoints: endpoint_counts.offline,
        warning_endpoints: endpoint_counts.warning,
        critical_endpoints: endpoint_counts.critical,
        total_checks: check_counts.total,
        enabled_checks: check_counts.enabled,
        recent_results,
    }
}

// Endpoints
pub async fn endpoints_list(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> impl IntoResponse {
    let endpoint_list = endpoints::list_endpoints(&state.pool)
        .await
        .unwrap_or_default();

    let endpoints: Vec<EndpointView> = endpoint_list.into_iter().map(EndpointView::from).collect();

    EndpointsTemplate {
        title: "Endpoints".to_string(),
        endpoints,
    }
}

pub async fn endpoint_detail(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Response {
    let endpoint = match endpoints::get_endpoint_by_id(&state.pool, id).await {
        Ok(Some(e)) => e,
        _ => return Redirect::to("/endpoints").into_response(),
    };

    let snapshot = snapshots::get_latest_snapshot(&state.pool, id)
        .await
        .ok()
        .flatten();

    let latest_results = results::get_latest_results_for_endpoint(&state.pool, id)
        .await
        .unwrap_or_default();

    let check_results: Vec<CheckResultView> = latest_results
        .into_iter()
        .map(|r| CheckResultView {
            check_id: r.check_id,
            check_name: r.check_name,
            status: r.status.parse().unwrap_or(CheckStatus::Error),
            message: r.message,
            collected_at: r.collected_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        })
        .collect();

    EndpointDetailTemplate {
        title: format!("Endpoint: {}", endpoint.hostname),
        endpoint: EndpointView::from(endpoint),
        snapshot: snapshot.map(SnapshotView::from),
        check_results,
    }
    .into_response()
}

pub async fn endpoint_delete(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let _ = endpoints::delete_endpoint(&state.pool, id).await;
    Redirect::to("/endpoints")
}

// Checks
pub async fn checks_list(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> impl IntoResponse {
    let check_list = checks::list_checks(&state.pool).await.unwrap_or_default();

    let checks: Vec<CheckDefView> = check_list
        .into_iter()
        .map(|c| CheckDefView {
            id: c.id,
            name: c.name,
            description: c.description.unwrap_or_default(),
            check_type: c.check_type,
            severity: c.severity.parse().unwrap_or(Severity::Medium),
            enabled: c.enabled,
            updated_at: c.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        })
        .collect();

    ChecksTemplate {
        title: "Check Definitions".to_string(),
        checks,
    }
}

pub async fn check_new(_user: AuthenticatedUser) -> impl IntoResponse {
    CheckFormTemplate {
        title: "New Check".to_string(),
        check: None,
        parameters_json: "{}".to_string(),
    }
}

pub async fn check_edit(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Response {
    let check = match checks::get_check_by_id(&state.pool, id).await {
        Ok(Some(c)) => c,
        _ => return Redirect::to("/checks").into_response(),
    };

    CheckFormTemplate {
        title: format!("Edit Check: {}", check.name),
        check: Some(CheckDefView {
            id: check.id,
            name: check.name,
            description: check.description.unwrap_or_default(),
            check_type: check.check_type,
            severity: check.severity.parse().unwrap_or(Severity::Medium),
            enabled: check.enabled,
            updated_at: check.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }),
        parameters_json: serde_json::to_string_pretty(&check.parameters).unwrap_or_default(),
    }
    .into_response()
}

#[derive(Debug, Deserialize)]
pub struct CheckForm {
    pub name: String,
    pub description: String,
    pub check_type: String,
    pub parameters: String,
    pub severity: String,
    #[serde(default)]
    pub enabled: Option<String>,
}

pub async fn check_create(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Form(form): Form<CheckForm>,
) -> impl IntoResponse {
    let parameters: serde_json::Value = serde_json::from_str(&form.parameters).unwrap_or_default();
    let severity: Severity = form.severity.parse().unwrap_or(Severity::Medium);
    let enabled = form.enabled.is_some();

    let _ = checks::create_check(
        &state.pool,
        &form.name,
        if form.description.is_empty() {
            None
        } else {
            Some(&form.description)
        },
        &form.check_type,
        parameters,
        severity,
        enabled,
    )
    .await;

    Redirect::to("/checks")
}

pub async fn check_update(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Form(form): Form<CheckForm>,
) -> impl IntoResponse {
    let parameters: serde_json::Value = serde_json::from_str(&form.parameters).unwrap_or_default();
    let severity: Severity = form.severity.parse().unwrap_or(Severity::Medium);
    let enabled = form.enabled.is_some();

    let _ = checks::update_check(
        &state.pool,
        id,
        &form.name,
        if form.description.is_empty() {
            None
        } else {
            Some(&form.description)
        },
        &form.check_type,
        parameters,
        severity,
        enabled,
    )
    .await;

    Redirect::to("/checks")
}

pub async fn check_delete(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let _ = checks::delete_check(&state.pool, id).await;
    Redirect::to("/checks")
}

// Reports
pub async fn reports(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> impl IntoResponse {
    let stats = results::get_result_stats(&state.pool)
        .await
        .unwrap_or(crate::db::results::ResultStats {
            total: 0,
            passed: 0,
            failed: 0,
            errors: 0,
        });

    ReportsTemplate {
        title: "Reports".to_string(),
        total_results: stats.total,
        passed: stats.passed,
        failed: stats.failed,
        errors: stats.errors,
    }
}

// Auth
pub async fn login_page() -> impl IntoResponse {
    LoginTemplate {
        title: "Login".to_string(),
        error: None,
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

pub async fn login_submit(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> Response {
    let user = match users::get_user_by_username(&state.pool, &form.username).await {
        Ok(Some(u)) => u,
        _ => {
            return LoginTemplate {
                title: "Login".to_string(),
                error: Some("Invalid username or password".to_string()),
            }
            .into_response();
        }
    };

    if !verify_password(&form.password, &user.password_hash) {
        return LoginTemplate {
            title: "Login".to_string(),
            error: Some("Invalid username or password".to_string()),
        }
        .into_response();
    }

    let session = Session::new(&user);
    let cookie = create_session_cookie(&session);
    let jar = jar.add(cookie);

    (jar, Redirect::to("/")).into_response()
}

pub async fn logout(jar: CookieJar) -> impl IntoResponse {
    let jar = jar.add(clear_session_cookie());
    (jar, Redirect::to("/login"))
}

// Setup - create initial admin user
pub async fn setup_page(State(state): State<AppState>) -> Response {
    // Check if any users exist
    let count = users::user_count(&state.pool).await.unwrap_or(0);
    if count > 0 {
        return Redirect::to("/login").into_response();
    }

    Html(r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Setup - Endpoint Assessment</title>
            <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/css/bootstrap.min.css" rel="stylesheet">
        </head>
        <body class="bg-light">
            <div class="container mt-5">
                <div class="row justify-content-center">
                    <div class="col-md-6">
                        <div class="card">
                            <div class="card-header">
                                <h4>Initial Setup</h4>
                            </div>
                            <div class="card-body">
                                <p>Create the initial admin user:</p>
                                <form method="POST" action="/setup">
                                    <div class="mb-3">
                                        <label class="form-label">Username</label>
                                        <input type="text" name="username" class="form-control" required>
                                    </div>
                                    <div class="mb-3">
                                        <label class="form-label">Password</label>
                                        <input type="password" name="password" class="form-control" required>
                                    </div>
                                    <button type="submit" class="btn btn-primary">Create Admin User</button>
                                </form>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </body>
        </html>
    "#).into_response()
}

#[derive(Debug, Deserialize)]
pub struct SetupForm {
    pub username: String,
    pub password: String,
}

pub async fn setup_submit(
    State(state): State<AppState>,
    Form(form): Form<SetupForm>,
) -> Response {
    // Check if any users exist
    let count = users::user_count(&state.pool).await.unwrap_or(0);
    if count > 0 {
        return Redirect::to("/login").into_response();
    }

    let password_hash = match hash_password(&form.password) {
        Ok(h) => h,
        Err(_) => return Redirect::to("/setup").into_response(),
    };

    let _ = users::create_user(&state.pool, &form.username, &password_hash, AdminRole::Admin).await;

    Redirect::to("/login").into_response()
}
