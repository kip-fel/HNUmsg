use anyhow::{anyhow, Result};
use tokio::sync::{Mutex, RwLock};

use hnu_query::{
    cas::{login::CasToken, tfa::TFAToken},
    xgxt::{
        get_person_info,
        login::XgxtToken,
        personal_info::{Dormitory, PersonalInfo},
    },
};

use crate::config::Config;

use super::{
    auth::{self, LoginResult, SmsVerificationResult},
    dorm::validate_dormitory,
    electricity,
};

enum SessionState {
    LoggedOut,

    WaitingForSms {
        token: TFAToken,
        phone: String,
    },

    Ready {
        cas_token: CasToken,
        xgxt_token: XgxtToken,
        personal_info: PersonalInfo,
    },
}

pub struct SessionManager {
    config: Config,

    state: RwLock<SessionState>,

    auth_lock: Mutex<()>,
}

impl SessionManager {
    pub fn new(config: Config) -> Self {
        Self {
            config,

            state: RwLock::new(SessionState::LoggedOut),

            auth_lock: Mutex::new(()),
        }
    }

    pub async fn status(&self) -> String {
        let state = self.state.read().await;

        match &*state {
            SessionState::LoggedOut => {
                "未登录".to_string()
            }

            SessionState::WaitingForSms { phone, .. } => {
                format!(
                    "等待短信验证码，手机号：{}",
                    mask_phone(phone)
                )
            }

            SessionState::Ready {
                personal_info,
                ..
            } => {
                if let Some(dormitory) = &personal_info.dormitory {
                    let park =
                        dormitory.park().unwrap_or("未知园区");

                    let build =
                        dormitory.build().unwrap_or("未知楼栋");

                    format!(
                        "已登录，宿舍：{} / {} / {}",
                        park,
                        build,
                        dormitory.room()
                    )
                } else {
                    "已登录，但学工系统没有返回宿舍信息"
                        .to_string()
                }
            }
        }
    }

    pub async fn login(&self) -> Result<String> {
        let _guard = self.auth_lock.lock().await;

        {
            let state = self.state.read().await;

            if matches!(&*state, SessionState::Ready { .. }) {
                return Ok("当前 Session 仍然有效".to_string());
            }
        }

        let result = auth::login(&self.config).await?;

        match result {
            LoginResult::Ready(cas_token) => {
                self.finish_login(cas_token).await?;

                Ok("登录成功".to_string())
            }

            LoginResult::WaitingForSms {
                token,
                phone,
            } => {
                let mut state = self.state.write().await;

                *state = SessionState::WaitingForSms {
                    token,
                    phone: phone.clone(),
                };

                Ok(format!(
                    "需要短信验证码，验证码已发送到 {}",
                    mask_phone(&phone)
                ))
            }
        }
    }

    async fn finish_login(
        &self,
        cas_token: CasToken,
    ) -> Result<()> {
        let xgxt_token =
            XgxtToken::acquire_by_cas_login(&cas_token)
                .await
                .map_err(|e| anyhow!("{}", e))?;

        let personal_info =
            get_person_info(&xgxt_token)
                .await
                .map_err(|e| anyhow!("{}", e))?;

        if let Some(dormitory) = &personal_info.dormitory {
            validate_dormitory(dormitory)?;
        }

        let mut state = self.state.write().await;

        *state = SessionState::Ready {
            cas_token,
            xgxt_token,
            personal_info,
        };

        Ok(())
    }

    pub async fn verify_sms(
        &self,
        code: &str,
    ) -> Result<String> {
        let _guard = self.auth_lock.lock().await;

        let token = {
            let mut state = self.state.write().await;

            match std::mem::replace(
                &mut *state,
                SessionState::LoggedOut,
            ) {
                SessionState::WaitingForSms { token, .. } => {
                    token
                }

                other => {
                    *state = other;

                    return Err(anyhow!(
                        "当前没有等待短信验证码"
                    ));
                }
            }
        };

        match auth::verify_sms(token, code).await? {
            SmsVerificationResult::Success(cas_token) => {
                self.finish_login(cas_token).await?;

                Ok("短信验证成功，登录完成".to_string())
            }

            SmsVerificationResult::CodeError {
                token,
                phone,
            } => {
                let mut state = self.state.write().await;

                *state = SessionState::WaitingForSms {
                    token,
                    phone: phone.clone(),
                };

                Ok(format!(
                    "验证码错误。新的验证状态已保存，手机号：{}",
                    mask_phone(&phone)
                ))
            }

            SmsVerificationResult::Expired => {
                Ok(
                    "短信验证码已过期，请重新调用登录流程"
                        .to_string()
                )
            }
        }
    }

    pub async fn refresh(&self) -> Result<String> {
        let _guard = self.auth_lock.lock().await;

        {
            let mut state = self.state.write().await;
            *state = SessionState::LoggedOut;
        }

        let result = auth::login(&self.config).await?;

        match result {
            LoginResult::Ready(cas_token) => {
                self.finish_login(cas_token).await?;

                Ok("Session 已刷新，登录成功".to_string())
            }

            LoginResult::WaitingForSms {
                token,
                phone,
            } => {
                let mut state = self.state.write().await;

                *state = SessionState::WaitingForSms {
                    token,
                    phone: phone.clone(),
                };

                Ok(format!(
                    "刷新 Session 时需要短信验证码，验证码已发送到 {}",
                    mask_phone(&phone)
                ))
            }
        }
    }

    pub async fn electricity(&self) -> Result<String> {
        if let Some(result) = self.try_electricity().await? {
            return Ok(result);
        }

        self.login().await?;

        if let Some(result) = self.try_electricity().await? {
            return Ok(result);
        }

        Err(anyhow!(
            "当前无法查询电量。请检查认证状态或调用 hnu_auth_status。"
        ))
    }

    async fn try_electricity(&self) -> Result<Option<String>> {
        let dormitory: Option<Dormitory> = {
            let state = self.state.read().await;

            match &*state {
                SessionState::Ready {
                    personal_info,
                    ..
                } => {
                    personal_info.dormitory.clone()
                }

                _ => None,
            }
        };

        let Some(dormitory) = dormitory else {
            return Ok(None);
        };

        let result =
            electricity::get_electricity(dormitory).await?;

        Ok(Some(result))
    }

    pub async fn dormitory_info(&self) -> Result<String> {
        let state = self.state.read().await;

        match &*state {
            SessionState::Ready {
                personal_info,
                ..
            } => {
                let dormitory =
                    personal_info
                        .dormitory
                        .as_ref()
                        .ok_or_else(|| {
                            anyhow!(
                                "学工系统没有返回宿舍信息"
                            )
                        })?;

                let park =
                    dormitory.park().unwrap_or("未知");

                let build =
                    dormitory.build().unwrap_or("未知");

                Ok(format!(
                    "园区：{}\n楼栋：{}\n房间：{}\n原始信息：{}",
                    park,
                    build,
                    dormitory.room(),
                    dormitory.raw_dormitory()
                ))
            }

            _ => Err(anyhow!(
                "当前尚未完成登录"
            )),
        }
    }
}

fn mask_phone(phone: &str) -> String {
    let chars: Vec<char> = phone.chars().collect();

    if chars.len() < 7 {
        return "******".to_string();
    }

    let prefix: String = chars[..3].iter().collect();
    let suffix: String =
        chars[chars.len() - 4..].iter().collect();

    format!("{}****{}", prefix, suffix)
}