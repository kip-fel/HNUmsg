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

pub fn describe_dormitory(dormitory: &Dormitory) -> String {
    let park = dormitory.park().unwrap_or("未知园区");
    let build = dormitory.build().unwrap_or("未知楼栋");

    format!(
        "{} / {} / {}",
        park,
        build,
        dormitory.room()
    )
}