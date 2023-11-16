use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

use tokio_rustls::rustls::{NoClientAuth, ServerConfig};
use tokio_rustls::rustls::internal::pemfile;

fn load_tls_config() -> Result<Arc<ServerConfig>, Box<dyn Error>> {
    // 创建 ServerConfig 实例
    let mut config = ServerConfig::new(NoClientAuth::new());

    // 添加ALPN协议
    config.set_protocols(&[b"h2".to_vec(), b"http/1.1".to_vec()]);
    // 加载服务器的私钥和证书
    let cert_file = &mut BufReader::new(File::open("/path/to/cert.pem")?);
    let key_file = &mut BufReader::new(File::open("/path/to/key.pem")?);
    let cert_chain = pemfile::certs(cert_file).unwrap();
    let mut keys = pemfile::rsa_private_keys(key_file).unwrap();
    config.set_single_cert(cert_chain, keys.remove(0)).unwrap();

    // 使用配置创建 TLS 连接
    let tls_cfg = Arc::new(config);
    Ok(tls_cfg)
}