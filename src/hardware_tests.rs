use super::*; // Gives access to private functions from parent

#[cfg(test)]
pub fn mock_fetch_cpu_info() -> Result<String, std::io::Error> {
    return Ok(String::from(
        "model name      : Intel(R) Core(TM) Ultra 9 285H
         vendor_id       : GenuineIntel
         cpu cores       : 16
         siblings        : 16
        ",
    ));
}

#[cfg(test)]
pub fn mock_fetch_smt_info() -> Result<String, std::io::Error> {
    return Ok(String::from("0"));
}

#[cfg(test)]
pub fn mock_fetch_cpu_boost() -> Result<String, std::io::Error> {
    return Ok(String::from("1"));
}

#[test]
fn test_get_cpu() {
    assert_eq!(
        get_cpu(),
        CPUInfo {
            cpu_name: "Intel(R) Core(TM) Ultra 9 285H".to_string(),
            core_count: 16,
            thread_count: 16,
            vendor: CPUVendor::Intel,
            is_smt_enabled: false,
            is_turbo_boost_enabled: true
        }
    );
}
