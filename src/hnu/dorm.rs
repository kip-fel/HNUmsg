use anyhow::{bail, Result};
use hnu_query::xgxt::personal_info::Dormitory;

pub fn validate_dormitory(dormitory: &Dormitory) -> Result<()> {
    if !dormitory.successfully_parsed() {
        bail!(
            "学工系统返回了宿舍信息，但无法解析园区或楼栋。原始信息：{}",
            dormitory.raw_dormitory()
        );
    }

    Ok(())
}
