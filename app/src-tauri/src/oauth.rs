use crate::secrets;
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    PkceCodeChallenge, RedirectUrl, RefreshToken, Scope, TokenResponse, TokenUrl,
};
use std::time::{Duration, Instant};
use tauri_plugin_opener::OpenerExt;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::Mutex,
};
pub const GOOGLE_SCOPES: [&str; 2] = [
    "https://www.googleapis.com/auth/gmail.readonly",
    "https://www.googleapis.com/auth/calendar.readonly",
];
#[derive(Default)]
struct Session {
    token: Option<(String, Instant)>,
    email: Option<String>,
}
#[derive(Default)]
pub struct GoogleAuth {
    session: Mutex<Session>,
}
fn client() -> Result<
    oauth2::basic::BasicClient<
        oauth2::EndpointSet,
        oauth2::EndpointNotSet,
        oauth2::EndpointNotSet,
        oauth2::EndpointNotSet,
        oauth2::EndpointSet,
    >,
    String,
> {
    let id = secrets::get("google.client_id")?.ok_or("Google 클라이언트를 먼저 저장하세요")?;
    let secret =
        secrets::get("google.client_secret")?.ok_or("Google 클라이언트를 먼저 저장하세요")?;
    Ok(BasicClient::new(ClientId::new(id))
        .set_client_secret(ClientSecret::new(secret))
        .set_auth_uri(
            AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".into())
                .map_err(|_| "인증 URL 오류")?,
        )
        .set_token_uri(
            TokenUrl::new("https://oauth2.googleapis.com/token".into())
                .map_err(|_| "토큰 URL 오류")?,
        ))
}
fn http() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|_| "HTTP 초기화 실패".into())
}
impl GoogleAuth {
    pub fn is_connected(&self) -> bool {
        matches!(secrets::get("google.refresh_token"), Ok(Some(_)))
    }
    pub async fn email(&self) -> Option<String> {
        self.session.lock().await.email.clone()
    }
    pub async fn set_email(&self, email: String) {
        self.session.lock().await.email = Some(email);
    }
    pub async fn access_token(&self) -> Result<String, String> {
        let mut session = self.session.lock().await;
        if let Some((token, expiry)) = &session.token {
            if *expiry > Instant::now() + Duration::from_secs(60) {
                return Ok(token.clone());
            }
        }
        let refresh = secrets::get("google.refresh_token")?.ok_or("다시 로그인 필요")?;
        let response = client()?
            .exchange_refresh_token(&RefreshToken::new(refresh))
            .request_async(&http()?)
            .await
            .map_err(|_| "다시 로그인 필요")?;
        if let Some(refresh) = response.refresh_token() {
            secrets::set("google.refresh_token", refresh.secret())?;
        }
        let token = response.access_token().secret().clone();
        let expiry = Instant::now() + response.expires_in().unwrap_or(Duration::from_secs(3600));
        session.token = Some((token.clone(), expiry));
        Ok(token)
    }
    pub async fn connect(&self, app: &tauri::AppHandle) -> Result<(), String> {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|_| "로그인 리스너 시작 실패")?;
        let port = listener
            .local_addr()
            .map_err(|_| "로그인 주소 오류")?
            .port();
        let client = client()?.set_redirect_uri(
            RedirectUrl::new(format!("http://127.0.0.1:{port}/callback"))
                .map_err(|_| "로그인 주소 오류")?,
        );
        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
        let mut request = client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(challenge)
            .add_extra_param("access_type", "offline")
            .add_extra_param("prompt", "consent");
        for scope in GOOGLE_SCOPES {
            request = request.add_scope(Scope::new(scope.into()));
        }
        let (url, state) = request.url();
        app.opener()
            .open_url(url.as_str(), None::<&str>)
            .map_err(|_| "브라우저 열기 실패")?;
        let code=tokio::time::timeout(Duration::from_secs(300),async {
            loop {
                let (mut socket,_)=listener.accept().await.map_err(|_|"로그인 콜백 실패".to_string())?;
                let mut bytes=Vec::new();let mut chunk=[0u8;1024];
                loop {let n=tokio::time::timeout(Duration::from_secs(5),socket.read(&mut chunk)).await.map_err(|_|"로그인 콜백 시간 초과")?.map_err(|_|"로그인 콜백 읽기 실패")?;if n==0{break;}bytes.extend_from_slice(&chunk[..n]);if bytes.windows(4).any(|b|b==b"\r\n\r\n")||bytes.len()>8192{break;}}
                let request=String::from_utf8_lossy(&bytes);let target=request.lines().next().and_then(|line|line.strip_prefix("GET ")).and_then(|line|line.split_whitespace().next()).unwrap_or("");
                let parsed=oauth2::url::Url::parse(&format!("http://127.0.0.1:{port}{target}")).map_err(|_|"로그인 콜백 오류")?;
                let params:std::collections::HashMap<_,_>=parsed.query_pairs().into_owned().collect();
                let valid=parsed.path()=="/callback" && params.get("state").is_some_and(|v|v==state.secret());
                let body=if valid{"앱으로 돌아가세요"}else{"잘못된 로그인 요청"};let status=if valid{"200 OK"}else{"400 Bad Request"};
                let reply=format!("HTTP/1.1 {status}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len());let _=socket.write_all(reply.as_bytes()).await;
                if !valid {continue;}
                return params.get("code").cloned().ok_or_else(||"로그인이 취소되었습니다".to_string());
            }
        }).await.map_err(|_|"로그인 시간 초과 (5분)")??;
        drop(listener);
        let response = client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(verifier)
            .request_async(&http()?)
            .await
            .map_err(|_| "로그인 토큰 교환 실패")?;
        let refresh = response
            .refresh_token()
            .ok_or("오프라인 접근 권한을 승인하세요")?;
        secrets::set("google.refresh_token", refresh.secret())?;
        let mut session = self.session.lock().await;
        session.email = None;
        session.token = Some((
            response.access_token().secret().clone(),
            Instant::now() + response.expires_in().unwrap_or(Duration::from_secs(3600)),
        ));
        Ok(())
    }
    pub async fn disconnect(&self) -> Result<(), String> {
        secrets::delete("google.refresh_token")?;
        *self.session.lock().await = Session::default();
        Ok(())
    }
}
