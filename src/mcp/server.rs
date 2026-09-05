use std::sync::Arc;

use rmcp::{
    handler::server::{
        tool::ToolRouter,
        wrapper::Parameters,
    },
    tool,
    tool_handler,
    tool_router,
    ErrorData as McpError,
    ServerHandler,
};

use serde::Deserialize;

use crate::hnu::session::SessionManager;

#[derive(Clone)]
pub struct HnuDormServer {
    session: Arc<SessionManager>,
    tool_router: ToolRouter<Self>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SmsCodeRequest {
    /// 湖南大学统一身份认证发送到绑定手机上的短信验证码
    pub code: String,
}

#[tool_router]
impl HnuDormServer {
    pub fn new(session: Arc<SessionManager>) -> Self {
        Self {
            session,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        name = "get_dorm_electricity",
        description = "查询当前账号绑定宿舍的实时剩余电量"
    )]
    async fn get_dorm_electricity(
        &self,
    ) -> Result<String, McpError> {
        self.session
            .electricity()
            .await
            .map_err(|e| McpError::internal_error(
                e.to_string(),
                None,
            ))
    }

    #[tool(
        name = "hnu_auth_status",
        description = "查询湖南大学账号当前认证状态"
    )]
    async fn hnu_auth_status(
        &self,
    ) -> Result<String, McpError> {
        Ok(self.session.status().await)
    }

    #[tool(
        name = "complete_hnu_sms_verification",
        description = "提交湖南大学统一身份认证短信验证码，完成双因素认证"
    )]
    async fn complete_hnu_sms_verification(
        &self,
        Parameters(request): Parameters<SmsCodeRequest>,
    ) -> Result<String, McpError> {
        self.session
            .verify_sms(&request.code)
            .await
            .map_err(|e| McpError::invalid_params(
                e.to_string(),
                None,
            ))
    }

    #[tool(
        name = "refresh_hnu_session",
        description = "重新登录湖南大学账号并刷新当前 Session"
    )]
    async fn refresh_hnu_session(
        &self,
    ) -> Result<String, McpError> {
        self.session
            .refresh()
            .await
            .map_err(|e| McpError::internal_error(
                e.to_string(),
                None,
            ))
    }

    #[tool(
        name = "get_dormitory_info",
        description = "查询当前账号在湖南大学学工系统中的宿舍信息"
    )]
    async fn get_dormitory_info(
        &self,
    ) -> Result<String, McpError> {
        self.session
            .dormitory_info()
            .await
            .map_err(|e| McpError::internal_error(
                e.to_string(),
                None,
            ))
    }
}

#[tool_handler]
impl ServerHandler for HnuDormServer {}