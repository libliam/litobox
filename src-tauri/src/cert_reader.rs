use serde::{Deserialize, Serialize};
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

// ponytail: debug 模式输出日志到 stderr，release 模式编译时移除（零开销）
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            eprintln!($($arg)*)
        }
    };
}

// ============ 数据结构 ============

#[derive(Debug, Clone, Serialize)]
pub struct CertInfo {
    pub subject: String,          // 主题（CN=xxx, O=xxx, ...）
    pub issuer: String,           // 颁发者
    pub not_before: String,       // 生效日期 (yyyy-MM-dd HH:mm:ss)
    pub not_after: String,        // 到期日期 (yyyy-MM-dd HH:mm:ss)
    pub serial_number: String,    // 序列号
    pub thumbprint: String,       // SHA1 指纹
    pub store_name: String,       // 来源存储 (Personal/Root/CA)
    pub has_private_key: bool,    // 是否有私钥
    pub is_expired: bool,         // 是否已过期
}

#[derive(Debug, Clone, Serialize)]
pub struct CertStoreList {
    pub personal: Vec<CertInfo>,
    pub root: Vec<CertInfo>,
    pub ca: Vec<CertInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CertDetail {
    pub subject: String,
    pub issuer: String,
    pub not_before: String,
    pub not_after: String,
    pub serial_number: String,
    pub version: String,
    pub thumbprint: String,
    pub thumbprint_sha256: String,
    pub san: Vec<String>,
    pub key_usage: Vec<String>,
    pub enhanced_key_usage: Vec<String>,
    pub basic_constraints: String,
    pub signature_algorithm: String,
    pub public_key: String,
    pub raw_pem: String,
    pub is_expired: bool,
    pub days_until_expiry: i64,
}

// ============ PowerShell 反序列化中间结构体 ============

#[derive(Debug, Deserialize)]
struct PsCertInfo {
    Subject: Option<String>,
    Issuer: Option<String>,
    NotBefore: Option<String>,
    NotAfter: Option<String>,
    SerialNumber: Option<String>,
    Thumbprint: Option<String>,
    StoreName: Option<String>,
    HasPrivateKey: Option<bool>,
    IsExpired: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct PsCertStoreRaw {
    Personal: Option<Vec<PsCertInfo>>,
    Root: Option<Vec<PsCertInfo>>,
    CA: Option<Vec<PsCertInfo>>,
}

#[derive(Debug, Deserialize)]
struct PsCertDetail {
    Subject: Option<String>,
    Issuer: Option<String>,
    NotBefore: Option<String>,
    NotAfter: Option<String>,
    SerialNumber: Option<String>,
    Version: Option<String>,
    Thumbprint: Option<String>,
    #[serde(rename = "ThumbprintSha256")]
    ThumbprintSha256: Option<String>,
    SAN: Option<Vec<String>>,
    KeyUsage: Option<Vec<String>>,
    EnhancedKeyUsage: Option<Vec<String>>,
    BasicConstraints: Option<String>,
    SignatureAlgorithm: Option<String>,
    PublicKey: Option<String>,
    RawPem: Option<String>,
    IsExpired: Option<bool>,
    DaysUntilExpiry: Option<f64>,
}

// ============ PowerShell 封装 ============

fn run_powershell(script: &str) -> Result<String, String> {
    let mut cmd = Command::new("powershell");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", script]);
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let output = cmd
        .output()
        .map_err(|e| format!("PowerShell 执行失败: {}", e))?;
    if !output.status.success() {
        let (stderr, _, _) = encoding_rs::GBK.decode(&output.stderr);
        return Err(format!("PowerShell 错误: {}", stderr));
    }
    let (text, _, _) = encoding_rs::GBK.decode(&output.stdout);
    Ok(text.into_owned())
}

// ============ PowerShell 脚本构建 ============

fn ps_escape(s: &str) -> String {
    s.replace('\'', "''")
}

fn build_store_query_script() -> String {
    r#"
$ErrorActionPreference = 'Stop'
try {
    $stores = @{
        Personal = 'Cert:\CurrentUser\My'
        Root     = 'Cert:\CurrentUser\Root'
        CA       = 'Cert:\CurrentUser\CA'
    }
    $result = @{}
    foreach ($key in $stores.Keys) {
        $path = $stores[$key]
        $certs = New-Object 'System.Collections.ArrayList'
        if (Test-Path $path) {
            $items = @(Get-ChildItem -Path $path)
            foreach ($item in $items) {
                $obj = New-Object PSObject -Property @{
                    Subject       = $item.Subject
                    Issuer        = $item.Issuer
                    NotBefore     = $item.NotBefore.ToString('yyyy-MM-dd HH:mm:ss')
                    NotAfter      = $item.NotAfter.ToString('yyyy-MM-dd HH:mm:ss')
                    SerialNumber  = $item.SerialNumber
                    Thumbprint    = $item.Thumbprint
                    StoreName     = $key
                    HasPrivateKey = $item.HasPrivateKey
                    IsExpired     = ($item.NotAfter -lt (Get-Date))
                }
                [void]$certs.Add($obj)
            }
        }
        $result[$key] = $certs
    }
    $result | ConvertTo-Json -Depth 3
} catch {
    Write-Output "{}"
}
"#.to_string()
}

fn build_detail_script(thumbprint: &str, store_name: &str) -> String {
    let thumbprint = ps_escape(thumbprint);
    let store_name = ps_escape(store_name);
    format!(
        r#"
$ErrorActionPreference = 'Stop'
try {{
    $storeMap = @{{
        Personal = 'Cert:\CurrentUser\My'
        Root     = 'Cert:\CurrentUser\Root'
        CA       = 'Cert:\CurrentUser\CA'
    }}
    $path = $storeMap['{store_name}']
    if (-not $path) {{ throw "Invalid store name: {store_name}" }}
    if (-not (Test-Path $path)) {{ throw "Store path not found: {store_name}" }}

    $cert = $null
    $items = @(Get-ChildItem -Path $path)
    foreach ($item in $items) {{
        if ($item.Thumbprint -eq '{thumbprint}') {{
            $cert = $item
            break
        }}
    }}
    if (-not $cert) {{ throw "Certificate not found with thumbprint: {thumbprint}" }}

    # SAN
    $sanList = New-Object 'System.Collections.ArrayList'
    foreach ($ext in $cert.Extensions) {{
        if ($ext.Oid.FriendlyName -eq 'Subject Alternative Name') {{
            $sanText = $ext.Format($false)
            if ($sanText) {{
                $parts = $sanText -split ', '
                foreach ($p in $parts) {{ [void]$sanList.Add($p.Trim()) }}
            }}
        }}
    }}

    # Key Usage
    $kuList = New-Object 'System.Collections.ArrayList'
    foreach ($ext in $cert.Extensions) {{
        if ($ext.Oid.FriendlyName -eq 'Key Usage') {{
            $kuText = $ext.Format($true)
            if ($kuText) {{
                $parts = $kuText -split ', '
                foreach ($p in $parts) {{ [void]$kuList.Add($p.Trim()) }}
            }}
        }}
    }}

    # Enhanced Key Usage
    $ekuList = New-Object 'System.Collections.ArrayList'
    foreach ($ext in $cert.Extensions) {{
        if ($ext.Oid.FriendlyName -eq 'Enhanced Key Usage') {{
            $ekuText = $ext.Format($true)
            if ($ekuText) {{
                $parts = $ekuText -split ', '
                foreach ($p in $parts) {{ [void]$ekuList.Add($p.Trim()) }}
            }}
        }}
    }}

    # Basic Constraints
    $bcText = 'End Entity'
    foreach ($ext in $cert.Extensions) {{
        if ($ext.Oid.FriendlyName -eq 'Basic Constraints') {{
            $bcText = $ext.Format($true)
        }}
    }}

    # PEM
    $rawB64 = [System.Convert]::ToBase64String($cert.RawData)
    $pem = "-----BEGIN CERTIFICATE-----`r`n"
    for ($i = 0; $i -lt $rawB64.Length; $i += 64) {{
        $pem += $rawB64.Substring($i, [Math]::Min(64, $rawB64.Length - $i)) + "`r`n"
    }}
    $pem += "-----END CERTIFICATE-----"

    $result = New-Object PSObject -Property @{{
        Subject            = $cert.Subject
        Issuer             = $cert.Issuer
        NotBefore          = $cert.NotBefore.ToString('yyyy-MM-dd HH:mm:ss')
        NotAfter           = $cert.NotAfter.ToString('yyyy-MM-dd HH:mm:ss')
        SerialNumber       = $cert.SerialNumber
        Version            = "V$($cert.Version)"
        Thumbprint         = $cert.Thumbprint
        ThumbprintSha256   = $cert.GetCertHashString()
        SAN                = $sanList.ToArray()
        KeyUsage           = $kuList.ToArray()
        EnhancedKeyUsage   = $ekuList.ToArray()
        BasicConstraints   = $bcText
        SignatureAlgorithm = $cert.SignatureAlgorithm.FriendlyName
        PublicKey          = "$($cert.PublicKey.Key.KeySize)-bit $($cert.PublicKey.Oid.FriendlyName)"
        RawPem             = $pem
        IsExpired          = ($cert.NotAfter -lt (Get-Date))
        DaysUntilExpiry    = [math]::Floor(($cert.NotAfter - (Get-Date)).TotalDays)
    }}
    $result | ConvertTo-Json -Depth 3
}} catch {{
    Write-Host "ERROR: $($_.Exception.Message)"
    exit 1
}}
"#
    )
}

fn build_file_parse_script(file_path: &str, password: &Option<String>) -> String {
    let escaped_path = ps_escape(file_path);
    let pwd_line = match password {
        Some(pwd) => format!(
            r#"$cert = New-Object System.Security.Cryptography.X509Certificates.X509Certificate2('{}', '{}')"#,
            escaped_path,
            ps_escape(pwd)
        ),
        None => format!(
            r#"$cert = New-Object System.Security.Cryptography.X509Certificates.X509Certificate2('{}')"#,
            escaped_path
        ),
    };
    format!(
        r#"
$ErrorActionPreference = 'Stop'
try {{
    {pwd_line}

    # SAN
    $sanList = New-Object 'System.Collections.ArrayList'
    foreach ($ext in $cert.Extensions) {{
        if ($ext.Oid.FriendlyName -eq 'Subject Alternative Name') {{
            $sanText = $ext.Format($false)
            if ($sanText) {{
                $parts = $sanText -split ', '
                foreach ($p in $parts) {{ [void]$sanList.Add($p.Trim()) }}
            }}
        }}
    }}

    # Key Usage
    $kuList = New-Object 'System.Collections.ArrayList'
    foreach ($ext in $cert.Extensions) {{
        if ($ext.Oid.FriendlyName -eq 'Key Usage') {{
            $kuText = $ext.Format($true)
            if ($kuText) {{
                $parts = $kuText -split ', '
                foreach ($p in $parts) {{ [void]$kuList.Add($p.Trim()) }}
            }}
        }}
    }}

    # Enhanced Key Usage
    $ekuList = New-Object 'System.Collections.ArrayList'
    foreach ($ext in $cert.Extensions) {{
        if ($ext.Oid.FriendlyName -eq 'Enhanced Key Usage') {{
            $ekuText = $ext.Format($true)
            if ($ekuText) {{
                $parts = $ekuText -split ', '
                foreach ($p in $parts) {{ [void]$ekuList.Add($p.Trim()) }}
            }}
        }}
    }}

    # Basic Constraints
    $bcText = 'End Entity'
    foreach ($ext in $cert.Extensions) {{
        if ($ext.Oid.FriendlyName -eq 'Basic Constraints') {{
            $bcText = $ext.Format($true)
        }}
    }}

    # PEM
    $rawB64 = [System.Convert]::ToBase64String($cert.RawData)
    $pem = "-----BEGIN CERTIFICATE-----`r`n"
    for ($i = 0; $i -lt $rawB64.Length; $i += 64) {{
        $pem += $rawB64.Substring($i, [Math]::Min(64, $rawB64.Length - $i)) + "`r`n"
    }}
    $pem += "-----END CERTIFICATE-----"

    $result = New-Object PSObject -Property @{{
        Subject            = $cert.Subject
        Issuer             = $cert.Issuer
        NotBefore          = $cert.NotBefore.ToString('yyyy-MM-dd HH:mm:ss')
        NotAfter           = $cert.NotAfter.ToString('yyyy-MM-dd HH:mm:ss')
        SerialNumber       = $cert.SerialNumber
        Version            = "V$($cert.Version)"
        Thumbprint         = $cert.Thumbprint
        ThumbprintSha256   = $cert.GetCertHashString()
        SAN                = $sanList.ToArray()
        KeyUsage           = $kuList.ToArray()
        EnhancedKeyUsage   = $ekuList.ToArray()
        BasicConstraints   = $bcText
        SignatureAlgorithm = $cert.SignatureAlgorithm.FriendlyName
        PublicKey          = "$($cert.PublicKey.Key.KeySize)-bit $($cert.PublicKey.Oid.FriendlyName)"
        RawPem             = $pem
        IsExpired          = ($cert.NotAfter -lt (Get-Date))
        DaysUntilExpiry    = [math]::Floor(($cert.NotAfter - (Get-Date)).TotalDays)
    }}
    $result | ConvertTo-Json -Depth 3
}} catch {{
    Write-Host "ERROR: $($_.Exception.Message)"
    exit 1
}}
"#
    )
}

// ============ 映射函数 ============

fn map_cert_info(ps: PsCertInfo) -> CertInfo {
    CertInfo {
        subject: ps.Subject.unwrap_or_default(),
        issuer: ps.Issuer.unwrap_or_default(),
        not_before: ps.NotBefore.unwrap_or_default(),
        not_after: ps.NotAfter.unwrap_or_default(),
        serial_number: ps.SerialNumber.unwrap_or_default(),
        thumbprint: ps.Thumbprint.unwrap_or_default(),
        store_name: ps.StoreName.unwrap_or_default(),
        has_private_key: ps.HasPrivateKey.unwrap_or(false),
        is_expired: ps.IsExpired.unwrap_or(false),
    }
}

fn map_cert_detail(ps: PsCertDetail) -> CertDetail {
    CertDetail {
        subject: ps.Subject.unwrap_or_default(),
        issuer: ps.Issuer.unwrap_or_default(),
        not_before: ps.NotBefore.unwrap_or_default(),
        not_after: ps.NotAfter.unwrap_or_default(),
        serial_number: ps.SerialNumber.unwrap_or_default(),
        version: ps.Version.unwrap_or_default(),
        thumbprint: ps.Thumbprint.unwrap_or_default(),
        thumbprint_sha256: ps.ThumbprintSha256.unwrap_or_default(),
        san: ps.SAN.unwrap_or_default(),
        key_usage: ps.KeyUsage.unwrap_or_default(),
        enhanced_key_usage: ps.EnhancedKeyUsage.unwrap_or_default(),
        basic_constraints: ps.BasicConstraints.unwrap_or_default(),
        signature_algorithm: ps.SignatureAlgorithm.unwrap_or_default(),
        public_key: ps.PublicKey.unwrap_or_default(),
        raw_pem: ps.RawPem.unwrap_or_default(),
        is_expired: ps.IsExpired.unwrap_or(false),
        days_until_expiry: ps.DaysUntilExpiry.map(|d| d as i64).unwrap_or(0),
    }
}

// ============ Tauri 命令 ============

#[tauri::command]
pub fn read_cert_store() -> Result<CertStoreList, String> {
    debug_log!("[cert_reader] 开始读取证书存储");
    let script = build_store_query_script();
    let output = run_powershell(&script)?;
    let raw: PsCertStoreRaw =
        serde_json::from_str(&output).map_err(|e| format!("JSON 解析失败: {}", e))?;

    let personal: Vec<CertInfo> = raw
        .Personal
        .unwrap_or_default()
        .into_iter()
        .map(map_cert_info)
        .collect();
    let root: Vec<CertInfo> = raw
        .Root
        .unwrap_or_default()
        .into_iter()
        .map(map_cert_info)
        .collect();
    let ca: Vec<CertInfo> = raw
        .CA
        .unwrap_or_default()
        .into_iter()
        .map(map_cert_info)
        .collect();

    debug_log!(
        "[cert_reader] 证书存储读取完成: personal={}, root={}, ca={}",
        personal.len(),
        root.len(),
        ca.len()
    );
    Ok(CertStoreList {
        personal,
        root,
        ca,
    })
}

#[tauri::command]
pub fn get_cert_detail(thumbprint: String, store_name: String) -> Result<CertDetail, String> {
    debug_log!(
        "[cert_reader] 获取证书详情: store={}, thumbprint={}",
        &store_name,
        &thumbprint
    );
    let script = build_detail_script(&thumbprint, &store_name);
    let output = run_powershell(&script)?;
    let raw: PsCertDetail =
        serde_json::from_str(&output).map_err(|e| format!("JSON 解析失败: {}", e))?;
    Ok(map_cert_detail(raw))
}

#[tauri::command]
pub fn parse_cert_file(file_path: String, password: Option<String>) -> Result<CertDetail, String> {
    debug_log!("[cert_reader] 解析证书文件: {}", &file_path);
    let script = build_file_parse_script(&file_path, &password);
    let output = run_powershell(&script)?;
    let raw: PsCertDetail =
        serde_json::from_str(&output).map_err(|e| format!("JSON 解析失败: {}", e))?;
    Ok(map_cert_detail(raw))
}

// ============ 单元测试 ============

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ps_escape() {
        assert_eq!(ps_escape("hello"), "hello");
        assert_eq!(ps_escape("it's"), "it''s");
        assert_eq!(ps_escape(""), "");
    }

    #[test]
    fn test_cert_info_serialization() {
        let info = CertInfo {
            subject: "CN=test".into(),
            issuer: "CN=ca".into(),
            not_before: "2024-01-01 00:00:00".into(),
            not_after: "2025-01-01 00:00:00".into(),
            serial_number: "00:11:22".into(),
            thumbprint: "ABCDEF".into(),
            store_name: "Personal".into(),
            has_private_key: true,
            is_expired: false,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("CN=test"));
        assert!(json.contains("has_private_key"));
    }

    #[test]
    fn test_cert_detail_serialization() {
        let detail = CertDetail {
            subject: "CN=test".into(),
            issuer: "CN=ca".into(),
            not_before: "2024-01-01 00:00:00".into(),
            not_after: "2025-01-01 00:00:00".into(),
            serial_number: "00:11:22".into(),
            version: "V3".into(),
            thumbprint: "ABCDEF".into(),
            thumbprint_sha256: "SHA256ABC".into(),
            san: vec!["DNS:*.example.com".into()],
            key_usage: vec!["Digital Signature".into(), "Key Encipherment".into()],
            enhanced_key_usage: vec!["Server Authentication".into()],
            basic_constraints: "End Entity".into(),
            signature_algorithm: "sha256RSA".into(),
            public_key: "2048-bit RSA".into(),
            raw_pem: "-----BEGIN CERTIFICATE-----\n...\n-----END CERTIFICATE-----".into(),
            is_expired: false,
            days_until_expiry: 180,
        };
        let json = serde_json::to_string(&detail).unwrap();
        assert!(json.contains("CN=test"));
        assert!(json.contains("thumbprint_sha256"));
        assert!(json.contains("days_until_expiry"));
    }

    #[test]
    fn test_build_store_query_script() {
        let script = build_store_query_script();
        assert!(script.contains("Cert:\\CurrentUser\\My"));
        assert!(script.contains("Cert:\\CurrentUser\\Root"));
        assert!(script.contains("Cert:\\CurrentUser\\CA"));
        assert!(!script.contains("ForEach-Object"));
        assert!(!script.contains("[PSCustomObject]"));
    }

    #[test]
    fn test_build_detail_script() {
        let script = build_detail_script("ABCDEF", "Personal");
        assert!(script.contains("ABCDEF"));
        assert!(script.contains("Personal"));
        assert!(!script.contains("Where-Object"));
    }

    #[test]
    fn test_build_file_parse_script() {
        let script = build_file_parse_script("C:\\test.cer", &None);
        assert!(script.contains("test.cer"));
        assert!(!script.contains("Where-Object"));
    }
}