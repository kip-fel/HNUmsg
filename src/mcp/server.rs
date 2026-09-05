use std::sync::Arc;

use rmcp::{
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
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

/// 湖南大学统一身份认证登录参数
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LoginRequest {
    /// 湖南大学统一身份认证学号
    pub stu_id: String,

    /// 湖南大学统一身份认证密码
    pub password: String,
}

/// 短信验证码参数
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SmsCodeRequest {
    /// 收到的短信验证码
    pub code: String,
}

#[tool_router]
impl HnuDormServer {
    pub fn new(session: Arc<SessionManager>) -> Self {
        let tool_router = Self::tool_router();



        Self {
            session,
            tool_router,
        }
    }

    /// 使用学号和密码登录湖南大学统一身份认证。
    ///
    /// 如果账号启用了短信二次验证，会自动发送短信，
    /// 然后需要调用 complete_hnu_sms_verification 完成登录。
    #[tool(
        name = "hnu_login",
        description = "使用湖南大学统一身份认证登录。需要提供学号和密码。如果账号启用了短信二次验证，将自动发送短信验证码，然后调用 complete_hnu_sms_verification 完成登录"
    )]
    async fn hnu_login(
        &self,
        Parameters(request): Parameters<LoginRequest>,
    ) -> Result<String, McpError> {
        self.session
            .login(&request.stu_id, &request.password)
            .await
            .map_err(|e| {
                McpError::invalid_params(
                    e.to_string(),
                    None,
                )
            })
    }

    /// 查询当前认证状态。
    #[tool(
        name = "hnu_auth_status",
        description = "查询当前湖南大学账号的登录和短信验证状态"
    )]
    async fn hnu_auth_status(
        &self,
    ) -> Result<String, McpError> {
        Ok(self.session.status().await)
    }

    /// 完成短信二次验证。
    #[tool(
        name = "complete_hnu_sms_verification",
        description = "提交湖南大学统一身份认证发送的短信验证码，完成登录"
    )]
    async fn complete_hnu_sms_verification(
        &self,
        Parameters(request): Parameters<SmsCodeRequest>,
    ) -> Result<String, McpError> {
        self.session
            .verify_sms(&request.code)
            .await
            .map_err(|e| {
                McpError::invalid_params(
                    e.to_string(),
                    None,
                )
            })
    }

    /// 查询宿舍信息。
    #[tool(
        name = "get_dormitory_info",
        description = "查询当前登录湖南大学账号对应的宿舍园区、楼栋、房间以及原始宿舍信息"
    )]
    async fn get_dormitory_info(
        &self,
    ) -> Result<String, McpError> {
        self.session
            .dormitory_info()
            .await
            .map_err(|e| {
                McpError::invalid_request(
                    e.to_string(),
                    None,
                )
            })
    }

    /// 查询宿舍电费。
    #[tool(
        name = "get_dorm_electricity",
        description = "查询当前登录湖南大学账号对应宿舍的剩余电量"
    )]
    async fn get_dorm_electricity(
        &self,
    ) -> Result<String, McpError> {
        self.session
            .electricity()
            .await
            .map_err(|e| {
                McpError::invalid_request(
                    e.to_string(),
                    None,
                )
            })
    }

    /// 使用新的学号和密码重新登录。
    ///
    /// 如果账号需要短信验证，会自动发送新的验证码。
    #[tool(
        name = "refresh_hnu_session",
        description = "清除当前湖南大学登录状态，并使用提供的新学号和密码重新登录"
    )]
    async fn refresh_hnu_session(
        &self,
        Parameters(request): Parameters<LoginRequest>,
    ) -> Result<String, McpError> {
        self.session
            .refresh(&request.stu_id, &request.password)
            .await
            .map_err(|e| {
                McpError::invalid_params(
                    e.to_string(),
                    None,
                )
            })
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for HnuDormServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            ..Default::default()
        }
    }
}