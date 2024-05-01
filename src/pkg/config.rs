#![allow(dead_code)]
use serde;
use std::{fs::File, io::BufReader};

pub const USE_SSL: bool = false;

#[derive(serde::Deserialize)]
pub struct Config {
    pub version: u8,
    pub threads: u8,
    pub pid_file: String,
    pub error_log: String,
    pub upgrade_sock: String,
    pub port: u16,
    pub access_log: String,
    pub upstream: Vec<UpstreamNode>,
    pub prometheus: PrometheusConfig,
}

#[derive(serde::Deserialize)]
pub struct UpstreamNode {
    pub uri_prefix: String,
    pub addr: String,
}

#[derive(serde::Deserialize)]
pub struct PrometheusConfig {
    pub enabled: bool,
    pub port: u16
}

pub fn load_config_from_file(filename:&str) -> Result<Config,Box<dyn std::error::Error>> {
    // 读取 YAML 文件
    let file = File::open(filename)?;
    let reader = BufReader::new(file);

    // 使用 serde_yaml 解析 YAML 数据到 Config 结构体
    let config: Config = serde_yaml::from_reader(reader)?;
    Ok(config)
}