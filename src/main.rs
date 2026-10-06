#[cfg(target_os = "macos")]
use std::ffi::CString;
#[cfg(target_os = "linux")]
use std::ffi::CString;
#[cfg(target_os = "macos")]
use std::ffi::c_char;
use std::ffi::c_int;
#[cfg(target_os = "linux")]
use std::ffi::{CStr, c_char};
#[cfg(target_os = "macos")]
use std::ptr;

include!(concat!(env!("OUT_DIR"), "/logos.rs"));

unsafe extern "C" {
    fn gethostname(name: *mut u8, len: usize) -> c_int;
}

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn sysctlbyname(
        name: *const std::os::raw::c_char,
        oldp: *mut std::os::raw::c_void,
        oldlenp: *mut usize,
        newp: *const std::os::raw::c_void,
        newlen: usize,
    ) -> std::os::raw::c_int;
}

#[cfg(target_os = "linux")]
unsafe extern "C" {
    fn uname(buf: *mut UtsName) -> c_int;
}

#[cfg(target_os = "linux")]
#[repr(C)]
#[allow(dead_code)]
struct UtsName {
    sysname: [u8; 65],
    nodename: [u8; 65],
    release: [u8; 65],
    version: [u8; 65],
    machine: [u8; 65],
    domainname: [u8; 65],
}

fn get_hostname() -> String {
    let mut buffer = [0u8; 256];
    let res = unsafe { gethostname(buffer.as_mut_ptr(), buffer.len()) };

    if res == 0 {
        buffer[buffer.len() - 1] = 0;
        let len = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());
        String::from_utf8(buffer[..len].to_vec()).unwrap_or_else(|_| "unknown".to_string())
    } else {
        "unknown".to_string()
    }
}

fn get_os_info() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
            for line in content.lines() {
                if let Some(name) = line.strip_prefix("PRETTY_NAME=") {
                    return name.trim_matches('"').to_string();
                }
            }
        }
        "Linux".to_string()
    }

    #[cfg(target_os = "macos")]
    {
        let version = get_sysctl_str("kern.osproductversion").unwrap_or_default();

        if version.is_empty() {
            "macOS".to_string()
        } else {
            format!("macOS {version}")
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        std::env::consts::OS.to_string()
    }
}

#[cfg(target_os = "macos")]
fn get_sysctl_str(name: &str) -> Option<String> {
    let c_name = CString::new(name).ok()?;
    let mut len: usize = 0;

    unsafe {
        if sysctlbyname(c_name.as_ptr(), ptr::null_mut(), &mut len, ptr::null(), 0) != 0 || len == 0
        {
            return None;
        }
    }

    let mut buf = vec![0u8; len];

    unsafe {
        if sysctlbyname(
            c_name.as_ptr(),
            buf.as_mut_ptr() as *mut std::os::raw::c_void,
            &mut len,
            ptr::null(),
            0,
        ) != 0
        {
            return None;
        }
    }

    let c_str = std::ffi::CStr::from_bytes_until_nul(&buf).ok()?;
    c_str.to_str().map(|s| s.to_string()).ok()
}

#[cfg(target_os = "macos")]
fn get_sysctl_int(name: &str) -> Option<u32> {
    let c_name = CString::new(name).ok()?;
    let mut value: u32 = 0;
    let mut len = std::mem::size_of::<u32>();
    let res = unsafe {
        sysctlbyname(
            c_name.as_ptr(),
            &mut value as *mut u32 as *mut std::os::raw::c_void,
            &mut len,
            ptr::null(),
            0,
        )
    };
    if res != 0 {
        return None;
    }
    Some(value)
}

#[cfg(target_os = "macos")]
fn get_sysctl_u64(name: &str) -> Option<u64> {
    let c_name = CString::new(name).ok()?;
    let mut value: u64 = 0;
    let mut len = std::mem::size_of::<u64>();
    let res = unsafe {
        sysctlbyname(
            c_name.as_ptr(),
            &mut value as *mut u64 as *mut std::os::raw::c_void,
            &mut len,
            ptr::null(),
            0,
        )
    };
    if res != 0 {
        return None;
    }
    Some(value)
}

#[cfg(target_os = "linux")]
fn get_kernel() -> String {
    let mut uts: UtsName = unsafe { std::mem::zeroed() };
    let res = unsafe { uname(&mut uts as *mut UtsName) };

    if res == 0 {
        let release = unsafe { CStr::from_ptr(uts.release.as_ptr() as *const c_char) }
            .to_str()
            .unwrap_or("");
        let sysname = unsafe { CStr::from_ptr(uts.sysname.as_ptr() as *const c_char) }
            .to_str()
            .unwrap_or("");
        let sysname = if sysname.is_empty() { "Linux" } else { sysname };
        if !release.is_empty() {
            return format!("{sysname} {release}");
        }
    }

    if let Ok(release) = std::fs::read_to_string("/proc/sys/kernel/osrelease") {
        let release = release.trim();
        if !release.is_empty() {
            let sysname = std::fs::read_to_string("/proc/sys/kernel/ostype")
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|_| "Linux".to_string());
            let sysname = if sysname.is_empty() {
                "Linux".to_string()
            } else {
                sysname
            };
            return format!("{sysname} {release}");
        }
    }

    "unknown".to_string()
}

#[cfg(target_os = "macos")]
fn get_kernel() -> String {
    let release = get_sysctl_str("kern.osrelease").unwrap_or_default();
    if release.is_empty() {
        return "unknown".to_string();
    }
    let ostype = get_sysctl_str("kern.ostype").unwrap_or_default();
    let ostype = if ostype.is_empty() { "Darwin" } else { &ostype };
    format!("{ostype} {release}")
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_kernel() -> String {
    "unknown".to_string()
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn lookup_mac_model(id: &str) -> Option<&'static str> {
    match id {
        "MacBookPro1,1" => Some("MacBook Pro 15-inch Early 2006"),
        "MacBookPro1,2" => Some("MacBook Pro 17-inch 2006"),
        "MacBookPro2,1" => Some("MacBook Pro 17-inch Late 2006"),
        "MacBookPro2,2" => Some("MacBook Pro 15-inch Late 2006"),
        "MacBookPro3,1" => Some("MacBook Pro 15/17-inch Mid 2007"),
        "MacBookPro4,1" => Some("MacBook Pro 17/15-inch Early 2008"),
        "MacBookPro5,1" => Some("MacBook Pro 15-inch Late 2008"),
        "MacBookPro5,2" => Some("MacBook Pro 17-inch Mid/Early 2009"),
        "MacBookPro5,3" => Some("MacBook Pro 15-inch Mid 2009"),
        "MacBookPro5,5" => Some("MacBook Pro 13-inch Mid 2009"),
        "MacBookPro6,1" => Some("MacBook Pro 17-inch Mid 2010"),
        "MacBookPro6,2" => Some("MacBook Pro 15-inch Mid 2010"),
        "MacBookPro7,1" => Some("MacBook Pro 13-inch Mid 2010"),
        "MacBookPro8,1" => Some("MacBook Pro 13-inch 2011"),
        "MacBookPro8,2" => Some("MacBook Pro 15-inch 2011"),
        "MacBookPro8,3" => Some("MacBook Pro 17-inch 2011"),
        "MacBookPro9,1" => Some("MacBook Pro 15-inch Mid 2012"),
        "MacBookPro9,2" => Some("MacBook Pro 13-inch Mid 2012"),
        "MacBookPro10,1" => Some("MacBook Pro Retina 15-inch Mid 2012/Early 2013"),
        "MacBookPro10,2" => Some("MacBook Pro Retina 13-inch Late 2012/Early 2013"),
        "MacBookPro11,1" => Some("MacBook Pro Retina 13-inch Late 2013/Mid 2014"),
        "MacBookPro11,2" => Some("MacBook Pro Retina 15-inch Late 2013/Mid 2014"),
        "MacBookPro11,3" => Some("MacBook Pro Retina 15-inch Late 2013/Mid 2014"),
        "MacBookPro11,4" => Some("MacBook Pro Retina 15-inch Mid 2015"),
        "MacBookPro11,5" => Some("MacBook Pro Retina 15-inch Mid 2015"),
        "MacBookPro12,1" => Some("MacBook Pro Retina 13-inch Early 2015"),
        "MacBookPro13,1" => Some("MacBook Pro 13-inch 2016 Two Thunderbolt 3 ports"),
        "MacBookPro13,2" => Some("MacBook Pro 13-inch 2016 Four Thunderbolt 3 ports"),
        "MacBookPro13,3" => Some("MacBook Pro 15-inch 2016"),
        "MacBookPro14,1" => Some("MacBook Pro 13-inch 2017 Two Thunderbolt 3 ports"),
        "MacBookPro14,2" => Some("MacBook Pro 13-inch 2017 Four Thunderbolt 3 ports"),
        "MacBookPro14,3" => Some("MacBook Pro 15-inch 2017"),
        "MacBookPro15,1" => Some("MacBook Pro 15-inch 2018/2019"),
        "MacBookPro15,2" => Some("MacBook Pro 13-inch 2018/2019 Four Thunderbolt 3 ports"),
        "MacBookPro15,3" => Some("MacBook Pro 15-inch 2019"),
        "MacBookPro15,4" => Some("MacBook Pro 13-inch 2019 Two Thunderbolt 3 ports"),
        "MacBookPro16,1" => Some("MacBook Pro 16-inch 2019"),
        "MacBookPro16,2" => Some("MacBook Pro 13-inch 2020 Four Thunderbolt 3 ports"),
        "MacBookPro16,3" => Some("MacBook Pro 13-inch 2020 Two Thunderbolt 3 ports"),
        "MacBookPro16,4" => Some("MacBook Pro 16-inch 2019"),
        "MacBookPro17,1" => Some("MacBook Pro 13-inch M1 2020"),
        "MacBookPro18,1" => Some("MacBook Pro 16-inch 2021"),
        "MacBookPro18,2" => Some("MacBook Pro 16-inch 2021"),
        "MacBookPro18,3" => Some("MacBook Pro 14-inch 2021"),
        "MacBookPro18,4" => Some("MacBook Pro 14-inch 2021"),
        "MacBookAir1,1" => Some("MacBook Air Early 2008"),
        "MacBookAir2,1" => Some("MacBook Air Mid 2009"),
        "MacBookAir3,1" => Some("MacBook Air 11-inch Late 2010"),
        "MacBookAir3,2" => Some("MacBook Air 13-inch Late 2010"),
        "MacBookAir4,1" => Some("MacBook Air 11-inch Mid 2011"),
        "MacBookAir4,2" => Some("MacBook Air 13-inch Mid 2011"),
        "MacBookAir5,1" => Some("MacBook Air 11-inch Mid 2012"),
        "MacBookAir5,2" => Some("MacBook Air 13-inch Mid 2012"),
        "MacBookAir6,1" => Some("MacBook Air 11-inch Mid 2013/Early 2014"),
        "MacBookAir6,2" => Some("MacBook Air 13-inch Mid 2013/Early 2014"),
        "MacBookAir7,1" => Some("MacBook Air 11-inch Early 2015"),
        "MacBookAir7,2" => Some("MacBook Air 13-inch Early 2015/2017"),
        "MacBookAir8,1" => Some("MacBook Air Retina 13-inch 2018"),
        "MacBookAir8,2" => Some("MacBook Air Retina 13-inch 2019"),
        "MacBookAir9,1" => Some("MacBook Air Retina 13-inch 2020"),
        "MacBookAir10,1" => Some("MacBook Air M1 2020"),
        "MacBook1,1" => Some("MacBook 13-inch Early 2006"),
        "MacBook2,1" => Some("MacBook 13-inch Mid 2007"),
        "MacBook3,1" => Some("MacBook 13-inch Late 2007"),
        "MacBook4,1" => Some("MacBook 13-inch Early 2008"),
        "MacBook5,1" => Some("MacBook 13-inch Late 2008"),
        "MacBook5,2" => Some("MacBook 13-inch Early/Mid 2009"),
        "MacBook6,1" => Some("MacBook 13-inch Late 2009"),
        "MacBook7,1" => Some("MacBook 13-inch Mid 2010"),
        "MacBook8,1" => Some("MacBook Retina 12-inch Early 2015"),
        "MacBook9,1" => Some("MacBook Retina 12-inch Early 2016"),
        "MacBook10,1" => Some("MacBook Retina 12-inch 2017"),
        "Macmini1,1" => Some("Mac mini Early 2006"),
        "Macmini2,1" => Some("Mac mini Mid 2007"),
        "Macmini3,1" => Some("Mac mini Early/Late 2009"),
        "Macmini4,1" => Some("Mac mini Mid 2010"),
        "Macmini5,1" => Some("Mac mini Mid 2011"),
        "Macmini5,2" => Some("Mac mini Mid 2011"),
        "Macmini6,1" => Some("Mac mini Late 2012"),
        "Macmini6,2" => Some("Mac mini Late 2012"),
        "Macmini7,1" => Some("Mac mini Mid 2014"),
        "Macmini8,1" => Some("Mac mini 2018"),
        "Macmini9,1" => Some("Mac mini M1 2020"),
        "MacPro1,1" => Some("Mac Pro 2006"),
        "MacPro2,1" => Some("Mac Pro Early 2007"),
        "MacPro3,1" => Some("Mac Pro Early 2008"),
        "MacPro4,1" => Some("Mac Pro Early 2009"),
        "MacPro5,1" => Some("Mac Pro Mid 2010 - Mid 2012"),
        "MacPro6,1" => Some("Mac Pro Late 2013"),
        "MacPro7,1" => Some("Mac Pro 2019"),
        "Xserve1,1" => Some("Xserve Late 2006"),
        "Xserve2,1" => Some("Xserve Early 2008"),
        "Xserve3,1" => Some("Xserve Early 2009"),
        "Mac13,1" => Some("Mac Studio M1 Max 2022 Two USB-C front ports"),
        "Mac13,2" => Some("Mac Studio M1 Ultra 2022 Two Thunderbolt 4 front ports"),
        "Mac14,2" => Some("MacBook Air M2 2022"),
        "Mac14,3" => Some("Mac mini M2 2023 Two Thunderbolt 4 ports"),
        "Mac14,5" => Some("MacBook Pro 14-inch 2023"),
        "Mac14,6" => Some("MacBook Pro 16-inch 2023"),
        "Mac14,7" => Some("MacBook Pro 13-inch M2 2022"),
        "Mac14,8" => Some("Mac Pro 2023"),
        "Mac14,9" => Some("MacBook Pro 14-inch 2023"),
        "Mac14,10" => Some("MacBook Pro 16-inch 2023"),
        "Mac14,12" => Some("Mac mini M2 Pro 2023 Four Thunderbolt 4 ports"),
        "Mac14,13" => Some("Mac Studio M2 Max 2023 Two USB-C front ports"),
        "Mac14,14" => Some("Mac Studio M2 Ultra 2023 Two Thunderbolt 4 front ports"),
        "Mac14,15" => Some("MacBook Air 15-inch M2 2023"),
        "Mac15,3" => Some("MacBook Pro 14-inch Nov 2023 Two Thunderbolt / USB 4 ports"),
        "Mac15,4" => Some("iMac 24-inch 2023 Two Thunderbolt / USB 4 ports"),
        "Mac15,5" => Some("iMac 24-inch 2023 Two Thunderbolt / USB 4 ports Two USB 3 ports"),
        "Mac15,6" => Some("MacBook Pro 14-inch Nov 2023 Three Thunderbolt 4 ports"),
        "Mac15,7" => Some("MacBook Pro 16-inch Nov 2023 Three Thunderbolt 4 ports"),
        "Mac15,8" => Some("MacBook Pro 14-inch Nov 2023 Three Thunderbolt 4 ports"),
        "Mac15,9" => Some("MacBook Pro 16-inch Nov 2023 Three Thunderbolt 4 ports"),
        "Mac15,10" => Some("MacBook Pro 14-inch Nov 2023 Three Thunderbolt 4 ports"),
        "Mac15,11" => Some("MacBook Pro 16-inch Nov 2023 Three Thunderbolt 4 ports"),
        "Mac15,12" => Some("MacBook Air 13-inch M3 2024"),
        "Mac15,13" => Some("MacBook Air 15-inch M3 2024"),
        "Mac15,14" => Some("Mac Studio M3 Ultra 2025"),
        "Mac16,1" => Some("MacBook Pro 14-inch 2024 Three Thunderbolt 4 ports"),
        "Mac16,2" => Some("iMac 24-inch 2024 Two Thunderbolt / USB 4 ports"),
        "Mac16,3" => Some("iMac 24-inch 2024 Four Thunderbolt / USB 4 ports"),
        "Mac16,5" => Some("MacBook Pro 16-inch 2024 Three Thunderbolt 5 ports"),
        "Mac16,6" => Some("MacBook Pro 14-inch 2024 Three Thunderbolt 5 ports"),
        "Mac16,7" => Some("MacBook Pro 16-inch 2024 Three Thunderbolt 5 ports"),
        "Mac16,8" => Some("MacBook Pro 14-inch 2024 Three Thunderbolt 5 ports"),
        "Mac16,9" => Some("Mac Studio M4 Max 2025"),
        "Mac16,10" => Some("Mac mini 2024"),
        "Mac16,11" => Some("Mac mini 2024"),
        "Mac16,12" => Some("MacBook Air 13-inch M4 2025"),
        "Mac16,13" => Some("MacBook Air 15-inch M4 2025"),
        "Mac17,2" => Some("MacBook Pro 14-inch M5 2025"),
        "Mac17,3" => Some("MacBook Air 13-inch M5 2026"),
        "Mac17,4" => Some("MacBook Air 15-inch M5 2026"),
        "Mac17,5" => Some("MacBook Neo 13-inch A18 Pro 2026"),
        "Mac17,6" => Some("MacBook Pro 16-inch M5 Max 2026"),
        "Mac17,7" => Some("MacBook Pro 14-inch M5 Max 2026"),
        "Mac17,8" => Some("MacBook Pro 16-inch M5 Pro 2026"),
        "Mac17,9" => Some("MacBook Pro 14-inch M5 Pro 2026"),
        "Mac17,14" => Some("Mac Studio M5 Max 2026"),
        "Mac17,15" => Some("Mac Studio M5 Ultra 2026"),
        "Mac17,16" => Some("Mac mini M5 Pro 2026"),
        "Mac18,5" => Some("Mac mini M6 2026"),
        "iMac4,1" => Some("iMac 17/20-inch Early 2006"),
        "iMac4,2" => Some("iMac 17-inch Mid 2006"),
        "iMac5,1" => Some("iMac 17/20-inch Late 2006"),
        "iMac5,2" => Some("iMac 17-inch 2007"),
        "iMac6,1" => Some("iMac 24-inch Late 2006"),
        "iMac7,1" => Some("iMac 20/24-inch Mid 2007"),
        "iMac8,1" => Some("iMac 20/24-inch Early 2008"),
        "iMac9,1" => Some("iMac 24/20-inch Early 2009"),
        "iMac10,1" => Some("iMac 27/21.5-inch Late 2009"),
        "iMac11,2" => Some("iMac 21.5-inch Mid 2010"),
        "iMac11,3" => Some("iMac 27-inch Mid 2010"),
        "iMac12,1" => Some("iMac 21.5-inch Mid 2011"),
        "iMac12,2" => Some("iMac 27-inch Mid 2011"),
        "iMac13,1" => Some("iMac 21.5-inch Late 2012"),
        "iMac13,2" => Some("iMac 27-inch Late 2012"),
        "iMac14,1" => Some("iMac 21.5-inch Late 2013"),
        "iMac14,2" => Some("iMac 27-inch Late 2013"),
        "iMac14,3" => Some("iMac 21.5-inch Late 2013"),
        "iMac14,4" => Some("iMac 21.5-inch Mid 2014"),
        "iMac15,1" => Some("iMac Retina 5K 27-inch Late 2014 - Mid 2015"),
        "iMac16,1" => Some("iMac 21.5-inch Late 2015"),
        "iMac16,2" => Some("iMac Retina 4K 21.5-inch Late 2015"),
        "iMac17,1" => Some("iMac Retina 5K 27-inch Late 2015"),
        "iMac18,1" => Some("iMac 21.5-inch 2017"),
        "iMac18,2" => Some("iMac Retina 4K 21.5-inch 2017"),
        "iMac18,3" => Some("iMac Retina 5K 27-inch 2017"),
        "iMac19,1" => Some("iMac Retina 5K 27-inch 2019"),
        "iMac19,2" => Some("iMac Retina 4K 21.5-inch 2019"),
        "iMac20,1" => Some("iMac Retina 5K 27-inch 2020"),
        "iMac20,2" => Some("iMac Retina 5K 27-inch 2020"),
        "iMac21,1" => Some("iMac 24-inch M1 2021 Two Thunderbolt / USB 4 ports Two USB 3 ports"),
        "iMac21,2" => Some("iMac 24-inch M1 2021 Two Thunderbolt / USB 4 ports"),
        "iMacPro1,1" => Some("iMac Pro 2017"),
        _ => None,
    }
}

#[cfg(target_os = "macos")]
fn get_device() -> String {
    let id = get_sysctl_str("hw.product")
        .filter(|s| !s.is_empty())
        .or_else(|| get_sysctl_str("hw.model").filter(|s| !s.is_empty()));
    let id = match id {
        Some(id) => id,
        None => return "unknown".to_string(),
    };
    match lookup_mac_model(&id) {
        Some(name) => name.to_string(),
        None => id,
    }
}

#[cfg(target_os = "linux")]
fn read_dmi_file(file: &str) -> Option<String> {
    for base in ["/sys/devices/virtual/dmi/id/", "/sys/class/dmi/id/"] {
        let mut path = String::with_capacity(base.len() + file.len());
        path.push_str(base);
        path.push_str(file);
        if let Ok(content) = std::fs::read_to_string(&path) {
            let value = content
                .trim_matches(|c: char| c.is_whitespace() || c == '\0')
                .to_string();
            if is_valid_dmi_value(&value) {
                return Some(value);
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn is_valid_dmi_value(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }
    !matches!(
        value,
        "To Be Filled By O.E.M."
            | "To be filled by O.E.M."
            | "Default string"
            | "Not Specified"
            | "Not Applicable"
            | "Unknown"
            | "None"
            | "System Product Name"
            | "System Version"
            | "Undefined"
    )
}

#[cfg(target_os = "linux")]
fn looks_like_model_code(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() < 5 || bytes.len() > 24 {
        return false;
    }
    let mut has_digit = false;
    let mut has_upper = false;
    for &b in bytes {
        match b {
            b'0'..=b'9' => has_digit = true,
            b'A'..=b'Z' => has_upper = true,
            b'-' | b'_' | b'.' => {}
            _ => return false,
        }
    }
    has_digit && has_upper
}

#[cfg(target_os = "linux")]
fn is_generic_dmi_word(value: &str) -> bool {
    matches!(
        value,
        "Laptop"
            | "Notebook"
            | "Desktop"
            | "Computer"
            | "System"
            | "PC"
            | "Tablet"
            | "Server"
            | "Workstation"
    )
}

#[cfg(target_os = "linux")]
fn get_device_from(
    name: Option<String>,
    version: Option<String>,
    family: Option<String>,
    vendor: Option<String>,
) -> Option<String> {
    let human = [&name, &family, &version]
        .into_iter()
        .flatten()
        .find(|value| !looks_like_model_code(value) && !is_generic_dmi_word(value))
        .cloned();
    let primary = human.or(name).or(family).or(version.clone())?;
    if primary.starts_with("Standard PC") {
        return Some(format!("KVM/QEMU {primary}"));
    }
    if (primary.starts_with("Mac") || primary.starts_with("iMac"))
        && vendor.as_deref() == Some("Apple Inc.")
    {
        if let Some(model) = lookup_mac_model(&primary) {
            return Some(model.to_string());
        }
    }
    match version {
        Some(value)
            if !value.is_empty()
                && value != primary
                && !primary.contains(value.as_str())
                && !looks_like_model_code(&value)
                && !is_generic_dmi_word(&value) =>
        {
            Some(format!("{primary} ({value})"))
        }
        _ => Some(primary),
    }
}

#[cfg(target_os = "linux")]
fn linux_device_fallback() -> String {
    for file in ["model", "banner-name"] {
        let mut path = String::from("/sys/firmware/devicetree/base/");
        path.push_str(file);
        if let Ok(content) = std::fs::read_to_string(&path) {
            let value = content
                .trim_matches(|c: char| c.is_whitespace() || c == '\0')
                .to_string();
            if !value.is_empty() {
                return value;
            }
        }
    }
    if std::env::var_os("WSL_DISTRO_NAME").is_some()
        || std::env::var_os("WSL_DISTRO").is_some()
        || std::env::var_os("WSL_INTEROP").is_some()
    {
        return "Windows Subsystem for Linux".to_string();
    }
    "unknown".to_string()
}

#[cfg(target_os = "linux")]
fn get_device() -> String {
    match get_device_from(
        read_dmi_file("product_name"),
        read_dmi_file("product_version"),
        read_dmi_file("product_family"),
        read_dmi_file("sys_vendor"),
    ) {
        Some(device) => device,
        None => linux_device_fallback(),
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_device() -> String {
    "unknown".to_string()
}

fn unit(value: u64, singular: &str, plural: &str) -> String {
    if value == 1 {
        format!("{value} {singular}")
    } else {
        format!("{value} {plural}")
    }
}

fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = secs % 86400 / 3600;
    let mins = secs % 3600 / 60;
    if days > 0 {
        format!(
            "{}, {}, {}",
            unit(days, "day", "days"),
            unit(hours, "hour", "hours"),
            unit(mins, "min", "mins")
        )
    } else if hours > 0 {
        format!(
            "{}, {}",
            unit(hours, "hour", "hours"),
            unit(mins, "min", "mins")
        )
    } else if mins > 0 {
        unit(mins, "min", "mins")
    } else {
        unit(secs, "sec", "secs")
    }
}

#[cfg(target_os = "linux")]
fn parse_uptime_content(content: &str) -> Option<u64> {
    content
        .split_whitespace()
        .next()?
        .split('.')
        .next()?
        .parse::<u64>()
        .ok()
}

#[cfg(target_os = "linux")]
fn get_uptime() -> String {
    match std::fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|content| parse_uptime_content(&content))
    {
        Some(secs) => format_uptime(secs),
        None => "unknown".to_string(),
    }
}

#[cfg(target_os = "macos")]
#[repr(C)]
#[allow(dead_code)]
struct Timeval {
    tv_sec: i64,
    tv_usec: i32,
}

#[cfg(target_os = "macos")]
fn get_boottime() -> Option<i64> {
    let name = CString::new("kern.boottime").ok()?;
    let mut boot = Timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut len = std::mem::size_of::<Timeval>();
    let res = unsafe {
        sysctlbyname(
            name.as_ptr(),
            &mut boot as *mut Timeval as *mut std::os::raw::c_void,
            &mut len,
            ptr::null(),
            0,
        )
    };
    if res != 0 {
        return None;
    }
    Some(boot.tv_sec)
}

#[cfg(target_os = "macos")]
fn get_uptime() -> String {
    let boot = match get_boottime() {
        Some(boot) => boot,
        None => return "unknown".to_string(),
    };
    let now = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(duration) => duration.as_secs() as i64,
        Err(_) => return "unknown".to_string(),
    };
    format_uptime(now.saturating_sub(boot).max(0) as u64)
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_uptime() -> String {
    "unknown".to_string()
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn shell_basename(path: &str) -> &str {
    path.rsplit('/')
        .find(|part| !part.is_empty())
        .unwrap_or(path)
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn is_known_shell(name: &str) -> bool {
    matches!(
        name,
        "sh" | "bash"
            | "dash"
            | "zsh"
            | "fish"
            | "nu"
            | "pwsh"
            | "powershell"
            | "ksh"
            | "mksh"
            | "yash"
            | "tcsh"
            | "csh"
            | "elvish"
            | "ion"
            | "xonsh"
    )
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn get_shell_from(parent: Option<String>, env_shell: Option<String>) -> Option<String> {
    if let Some(exe) = parent
        && is_known_shell(&exe)
    {
        return Some(exe);
    }
    if let Some(path) = env_shell
        && !path.is_empty()
    {
        return Some(shell_basename(&path).to_string());
    }
    None
}

#[cfg(target_os = "linux")]
fn ppid_from_stat(stat: &str) -> Option<u32> {
    stat.rsplit(')')
        .next()?
        .split_whitespace()
        .nth(1)?
        .parse()
        .ok()
}

#[cfg(target_os = "linux")]
fn parent_exe_basename() -> Option<String> {
    let stat = std::fs::read_to_string("/proc/self/stat").ok()?;
    let ppid = ppid_from_stat(&stat)?;
    let mut link = String::from("/proc/");
    link.push_str(&ppid.to_string());
    link.push_str("/exe");
    let target = std::fs::read_link(&link).ok()?;
    let target = target.to_str()?;
    let target = target.strip_suffix(" (deleted)").unwrap_or(target);
    Some(shell_basename(target).to_string())
}

#[cfg(target_os = "macos")]
fn parent_pid() -> u32 {
    unsafe extern "C" {
        fn getppid() -> c_int;
    }
    unsafe { getppid() }.max(0) as u32
}

#[cfg(target_os = "macos")]
fn exe_path_of_pid(pid: c_int) -> Option<String> {
    unsafe extern "C" {
        fn proc_pidpath(pid: c_int, buffer: *mut u8, buffersize: u32) -> c_int;
    }
    let mut buf = [0u8; 4096];
    let res = unsafe { proc_pidpath(pid, buf.as_mut_ptr(), buf.len() as u32) };
    if res <= 0 {
        return None;
    }
    let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    let path = std::str::from_utf8(&buf[..len]).ok()?;
    Some(path.to_string())
}

#[cfg(target_os = "macos")]
fn exe_basename_of_pid(pid: c_int) -> Option<String> {
    exe_path_of_pid(pid).map(|path| shell_basename(&path).to_string())
}

#[cfg(target_os = "macos")]
fn parent_exe_basename() -> Option<String> {
    exe_basename_of_pid(parent_pid() as c_int)
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn get_shell() -> String {
    match get_shell_from(parent_exe_basename(), std::env::var("SHELL").ok()) {
        Some(shell) => shell,
        None => "unknown".to_string(),
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_shell() -> String {
    "unknown".to_string()
}

#[cfg(target_os = "linux")]
fn init_name_from_comm(comm: &str, openrc_present: bool) -> Option<String> {
    let name = match comm {
        "systemd" => "systemd",
        "runit" => "runit",
        "s6-svscan" => "s6",
        "openrc-init" => "OpenRC",
        "init" if openrc_present => "OpenRC",
        "init" => "SysVinit",
        _ => comm,
    };
    if name.is_empty() {
        return None;
    }
    Some(name.to_string())
}

#[cfg(target_os = "linux")]
fn get_init() -> String {
    let comm = std::fs::read_to_string("/proc/1/comm")
        .map(|content| content.trim().to_string())
        .unwrap_or_default();
    let openrc_present = comm == "init" && std::path::Path::new("/run/openrc").exists();
    match init_name_from_comm(&comm, openrc_present) {
        Some(init) => init,
        None => "unknown".to_string(),
    }
}

#[cfg(target_os = "macos")]
fn get_init() -> String {
    match exe_basename_of_pid(1) {
        Some(init) if !init.is_empty() => init,
        _ => "unknown".to_string(),
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_init() -> String {
    "unknown".to_string()
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn is_terminal_skip(lower: &str) -> bool {
    matches!(
        lower,
        "sh" | "ash"
            | "bash"
            | "dash"
            | "zsh"
            | "fish"
            | "nu"
            | "pwsh"
            | "powershell"
            | "ksh"
            | "ksh93"
            | "mksh"
            | "pdksh"
            | "oksh"
            | "csh"
            | "tcsh"
            | "elvish"
            | "oil.ovm"
            | "xonsh"
            | "git-shell"
            | "yash"
            | "sudo"
            | "su"
            | "login"
            | "(login)"
            | "tmux"
            | "screen"
            | "zellij"
            | "script"
            | "proot"
            | "strace"
            | "gdb"
            | "lldb"
            | "lldb-mi"
            | "ltrace"
            | "perf"
            | "valgrind"
            | "time"
            | "run-parts"
            | "clifm"
            | "chezmoi"
            | "guake-wrapped"
            | "wfetch"
    ) || lower.starts_with("sshd")
        || lower.starts_with("tmux:")
        || lower.starts_with("screen-")
        || lower.starts_with("flatpak-")
        || lower.starts_with("Relay(")
        || lower.starts_with("command-not-")
        || lower.contains("debug")
        || lower.ends_with(".sh")
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn pretty_terminal_name(lower: &str) -> Option<&'static str> {
    if lower.starts_with("itermserver-") {
        return Some("iTerm2");
    }
    if lower.starts_with("gnome-terminal") {
        return Some("GNOME Terminal");
    }
    Some(match lower {
        "wezterm-gui" | "wezterm" => "WezTerm",
        "iterm.app" | "iterm2" | "iterm" => "iTerm2",
        "apple_terminal" => "Apple Terminal",
        #[cfg(target_os = "macos")]
        "terminal" => "Apple Terminal",
        "vscode" | "code" => "VS Code",
        "warpterminal" | "warp" => "Warp",
        "alacritty" => "Alacritty",
        "ghostty" => "Ghostty",
        "konsole" => "Konsole",
        "yakuake" => "Yakuake",
        "kgx" => "GNOME Console",
        "blackbox" => "Black Box",
        "urxvt" | "urxvtd" | "rxvt" | "rxvt-unicode" => "rxvt-unicode",
        "ptyxis-agent" | "ptyxis" => "Ptyxis",
        "tilix" => "Tilix",
        "terminator" => "Terminator",
        "deepin-terminal" => "Deepin Terminal",
        "qterminal" => "QTerminal",
        "mate-terminal" => "MATE Terminal",
        "xfce4-terminal" => "Xfce Terminal",
        "lxterminal" => "LXTerminal",
        "sakura" => "Sakura",
        "termite" => "Termite",
        "hyper" => "Hyper",
        "tabby" => "Tabby",
        "contour" => "Contour",
        "rio" => "Rio",
        "terminology" => "Terminology",
        "guake" => "Guake",
        "tilda" => "Tilda",
        "kitty" => "kitty",
        "xterm" => "xterm",
        "st" => "st",
        "foot" => "foot",
        _ => return None,
    })
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn terminal_display_name(raw: &str) -> String {
    let lower = raw.to_lowercase();
    match pretty_terminal_name(&lower) {
        Some(name) => name.to_string(),
        None => raw.to_string(),
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
struct TerminalEnv {
    kitty: bool,
    alacritty: bool,
    konsole: bool,
    gnome: bool,
    term_program: Option<String>,
    lc_terminal: Option<String>,
    terminal: Option<String>,
    term: Option<String>,
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn read_terminal_env() -> TerminalEnv {
    TerminalEnv {
        kitty: std::env::var_os("KITTY_PID").is_some()
            || std::env::var_os("KITTY_INSTALLATION_DIR").is_some(),
        alacritty: std::env::var_os("ALACRITTY_SOCKET").is_some()
            || std::env::var_os("ALACRITTY_LOG").is_some()
            || std::env::var_os("ALACRITTY_WINDOW_ID").is_some(),
        konsole: std::env::var_os("KONSOLE_VERSION").is_some(),
        gnome: std::env::var_os("GNOME_TERMINAL_SCREEN").is_some()
            || std::env::var_os("GNOME_TERMINAL_SERVICE").is_some(),
        term_program: std::env::var("TERM_PROGRAM").ok(),
        lc_terminal: std::env::var("LC_TERMINAL").ok(),
        terminal: std::env::var("TERMINAL").ok(),
        term: std::env::var("TERM").ok(),
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn terminal_from_env(env: &TerminalEnv) -> Option<String> {
    if env.kitty {
        return Some("kitty".to_string());
    }
    if env.alacritty {
        return Some("Alacritty".to_string());
    }
    if env.konsole {
        return Some("Konsole".to_string());
    }
    if env.gnome {
        return Some("GNOME Terminal".to_string());
    }
    if let Some(program) = env.term_program.as_deref()
        && !program.is_empty()
    {
        return Some(terminal_display_name(program));
    }
    if let Some(program) = env.lc_terminal.as_deref()
        && !program.is_empty()
    {
        return Some(terminal_display_name(program));
    }
    if let Some(value) = env.terminal.as_deref()
        && let Some(first) = value.split_whitespace().next()
        && !first.is_empty()
    {
        return Some(shell_basename(first).to_string());
    }
    if let Some(term) = env.term.as_deref()
        && !term.is_empty()
        && term != "linux"
        && term != "dumb"
        && term != "unknown"
        && !term.starts_with("screen")
        && !term.starts_with("tmux")
    {
        return Some(term.to_string());
    }
    None
}

#[cfg(target_os = "linux")]
fn proc_comm(pid: u32) -> Option<String> {
    let mut path = String::from("/proc/");
    path.push_str(&pid.to_string());
    path.push_str("/comm");
    let content = std::fs::read_to_string(&path).ok()?;
    let comm = content.trim().to_string();
    if comm.is_empty() {
        return None;
    }
    Some(comm)
}

#[cfg(target_os = "linux")]
fn proc_ppid_of(pid: u32) -> Option<u32> {
    let mut path = String::from("/proc/");
    path.push_str(&pid.to_string());
    path.push_str("/stat");
    let content = std::fs::read_to_string(&path).ok()?;
    ppid_from_stat(&content)
}

#[cfg(target_os = "linux")]
fn proc_exe_name(pid: u32) -> Option<String> {
    let mut path = String::from("/proc/");
    path.push_str(&pid.to_string());
    path.push_str("/exe");
    let target = std::fs::read_link(&path).ok()?;
    let target = target.to_str()?;
    let target = target.strip_suffix(" (deleted)").unwrap_or(target);
    Some(shell_basename(target).to_string())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
struct TerminalWalk {
    known: Option<String>,
    fallback: Option<String>,
}

#[cfg(target_os = "linux")]
fn walk_terminal_pid(start: u32, shell_skip: Option<&str>) -> TerminalWalk {
    let mut result = TerminalWalk {
        known: None,
        fallback: None,
    };
    let mut pid = start;
    for _ in 0..8 {
        if pid <= 1 {
            break;
        }
        let name = match proc_exe_name(pid).or_else(|| proc_comm(pid)) {
            Some(name) => name,
            None => break,
        };
        let lower = name.to_lowercase();
        if lower == "systemd" || lower == "init" || lower == "launchd" {
            break;
        }
        if is_terminal_skip(&lower) || Some(lower.as_str()) == shell_skip {
            match proc_ppid_of(pid) {
                Some(ppid) => pid = ppid,
                None => break,
            }
            continue;
        }
        if pretty_terminal_name(&lower).is_some() {
            result.known = Some(terminal_display_name(&name));
            break;
        }
        if result.fallback.is_none() {
            result.fallback = Some(name);
        }
        match proc_ppid_of(pid) {
            Some(ppid) => pid = ppid,
            None => break,
        }
    }
    result
}

#[cfg(target_os = "linux")]
fn get_terminal(shell: &str) -> String {
    let shell_lower = shell.to_lowercase();
    let shell_skip = if shell == "unknown" {
        None
    } else {
        Some(shell_lower.as_str())
    };
    let mut fallback: Option<String> = None;
    if let Ok(stat) = std::fs::read_to_string("/proc/self/stat") {
        if let Some(ppid) = ppid_from_stat(&stat) {
            let walk = walk_terminal_pid(ppid, shell_skip);
            if let Some(known) = walk.known {
                return known;
            }
            fallback = walk.fallback;
        }
    }
    if let Some(terminal) = terminal_from_env(&read_terminal_env()) {
        return terminal;
    }
    if let Some(name) = fallback {
        return name;
    }
    if let Some(terminal) = terminal_from_env(&read_terminal_env()) {
        return terminal;
    }
    if let Ok(target) = std::fs::read_link("/proc/self/fd/0") {
        if let Some(path) = target.to_str() {
            if path.starts_with("/dev/") {
                return path.to_string();
            }
        }
    }
    "unknown".to_string()
}

#[cfg(target_os = "macos")]
#[repr(C)]
#[allow(dead_code)]
struct ShortBsdInfo {
    pid: u32,
    ppid: u32,
    pgid: u32,
    status: u32,
    comm: [u8; 16],
    flags: u32,
    uid: u32,
    gid: u32,
    ruid: u32,
    rgid: u32,
    svuid: u32,
    svgid: u32,
    rfu: u32,
}

#[cfg(target_os = "macos")]
const _: () = assert!(std::mem::size_of::<ShortBsdInfo>() == 64);

#[cfg(target_os = "macos")]
fn proc_short_info(pid: u32) -> Option<ShortBsdInfo> {
    unsafe extern "C" {
        fn proc_pidinfo(
            pid: i32,
            flavor: i32,
            arg: u64,
            buffer: *mut ShortBsdInfo,
            buffersize: i32,
        ) -> i32;
    }
    let mut info: ShortBsdInfo = unsafe { std::mem::zeroed() };
    let res = unsafe {
        proc_pidinfo(
            pid as i32,
            13,
            0,
            &mut info,
            std::mem::size_of::<ShortBsdInfo>() as i32,
        )
    };
    if res <= 0 {
        return None;
    }
    Some(info)
}

#[cfg(target_os = "macos")]
fn short_comm(info: &ShortBsdInfo) -> &str {
    let len = info
        .comm
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(info.comm.len());
    std::str::from_utf8(&info.comm[..len]).unwrap_or("")
}

#[cfg(target_os = "macos")]
fn mac_bundle_name(pid: u32) -> Option<String> {
    let path = exe_path_of_pid(pid as c_int)?;
    let marker = ".app/Contents/MacOS/";
    let idx = path.find(marker)?;
    let app = path[..idx].rsplit('/').next()?;
    if app.is_empty() {
        return None;
    }
    Some(match app.to_lowercase().as_str() {
        "terminal" => "Apple Terminal".to_string(),
        "code" | "visual studio code" => "VS Code".to_string(),
        "iterm" => "iTerm2".to_string(),
        _ => app.to_string(),
    })
}

#[cfg(target_os = "macos")]
fn walk_terminal_pid(start: u32, shell_skip: Option<&str>) -> TerminalWalk {
    let mut result = TerminalWalk {
        known: None,
        fallback: None,
    };
    let mut pid = start;
    for _ in 0..8 {
        if pid <= 1 {
            break;
        }
        let info = match proc_short_info(pid) {
            Some(info) => info,
            None => break,
        };
        let comm = short_comm(&info).to_string();
        if comm.is_empty() {
            break;
        }
        let lower = comm.to_lowercase();
        if lower == "systemd" || lower == "init" || lower == "launchd" {
            break;
        }
        if is_terminal_skip(&lower) || Some(lower.as_str()) == shell_skip {
            pid = info.ppid;
            continue;
        }
        if pretty_terminal_name(&lower).is_some() {
            result.known = Some(terminal_display_name(&comm));
            break;
        }
        if result.fallback.is_none() {
            result.fallback = match mac_bundle_name(pid) {
                Some(app) => Some(app),
                None => Some(comm.clone()),
            };
        }
        pid = info.ppid;
    }
    result
}

#[cfg(target_os = "macos")]
fn get_terminal(shell: &str) -> String {
    let shell_lower = shell.to_lowercase();
    let shell_skip = if shell == "unknown" {
        None
    } else {
        Some(shell_lower.as_str())
    };
    let walk = walk_terminal_pid(parent_pid(), shell_skip);
    if let Some(known) = walk.known {
        return known;
    }
    if let Some(terminal) = terminal_from_env(&read_terminal_env()) {
        return terminal;
    }
    if let Some(name) = walk.fallback {
        return name;
    }
    "unknown".to_string()
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_terminal(_shell: &str) -> String {
    "unknown".to_string()
}

fn format_resolution(w: u32, h: u32, hz: u32) -> String {
    if hz > 0 {
        format!("{w}x{h} @ {hz}Hz")
    } else {
        format!("{w}x{h}")
    }
}

#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGGetOnlineDisplayList(max: u32, list: *mut u32, count: *mut u32) -> i32;
    fn CGDisplayCopyDisplayMode(display: u32) -> *mut std::ffi::c_void;
    fn CGDisplayModeGetPixelWidth(mode: *const std::ffi::c_void) -> usize;
    fn CGDisplayModeGetPixelHeight(mode: *const std::ffi::c_void) -> usize;
    fn CGDisplayModeGetRefreshRate(mode: *const std::ffi::c_void) -> f64;
    fn CGDisplayModeRelease(mode: *mut std::ffi::c_void);
    fn CGMainDisplayID() -> u32;
}

#[cfg(target_os = "macos")]
fn get_displays() -> Vec<(String, u32, u32, u32)> {
    let mut ids = [0u32; 16];
    let mut count = 0u32;
    if unsafe { CGGetOnlineDisplayList(ids.len() as u32, ids.as_mut_ptr(), &mut count) } != 0 {
        return Vec::new();
    }
    let mut displays = Vec::new();
    for &id in ids.iter().take(count as usize) {
        let mode = unsafe { CGDisplayCopyDisplayMode(id) };
        if mode.is_null() {
            continue;
        }
        let w = unsafe { CGDisplayModeGetPixelWidth(mode) };
        let h = unsafe { CGDisplayModeGetPixelHeight(mode) };
        let rate = unsafe { CGDisplayModeGetRefreshRate(mode) };
        unsafe { CGDisplayModeRelease(mode) };
        if w == 0 || h == 0 || w > u32::MAX as usize || h > u32::MAX as usize {
            continue;
        }
        let hz = if rate > 0.0 { rate.round() as u32 } else { 0 };
        displays.push((id, w as u32, h as u32, hz));
    }
    let main = unsafe { CGMainDisplayID() };
    displays.sort_by_key(|display| if display.0 == main { 0 } else { 1 });
    displays
        .into_iter()
        .map(|(_, w, h, hz)| (String::new(), w, h, hz))
        .collect()
}

#[cfg(target_os = "linux")]
fn parse_mode_line(line: &str) -> Option<(u32, u32)> {
    let (w, h) = line.trim().split_once('x')?;
    Some((w.parse().ok()?, h.parse().ok()?))
}

#[cfg(target_os = "linux")]
fn parse_edid_name(edid: &[u8]) -> Option<String> {
    if edid.len() < 128 {
        return None;
    }
    if edid[0..8] != [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00] {
        return None;
    }
    if edid
        .iter()
        .take(128)
        .fold(0u8, |sum, &b| sum.wrapping_add(b))
        != 0
    {
        return None;
    }
    for start in [54, 72, 90, 108] {
        let block = &edid[start..start + 18];
        if block[0] == 0 && block[1] == 0 && block[3] == 0xFC {
            if let Ok(text) = std::str::from_utf8(&block[5..18]) {
                let name = text
                    .trim_matches(|c: char| c.is_whitespace() || c == '\0')
                    .to_string();
                if !name.is_empty() {
                    return Some(name);
                }
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn drm_connector_label(entry: &str) -> &str {
    match entry.split_once('-') {
        Some((_, rest)) if !rest.is_empty() => rest,
        _ => entry,
    }
}

#[cfg(target_os = "linux")]
fn read_trimmed(path: &str) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|content| content.trim().to_string())
}

#[cfg(target_os = "linux")]
fn get_displays() -> Vec<(String, u32, u32, u32)> {
    let mut displays = Vec::new();
    let Ok(entries) = std::fs::read_dir("/sys/class/drm") else {
        return displays;
    };
    let mut names: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.contains('-'))
        .collect();
    names.sort();
    for name in names {
        let dir = String::from("/sys/class/drm/") + &name + "/";
        let enabled = read_trimmed(&(dir.clone() + "enabled")).as_deref() == Some("enabled");
        let connected = read_trimmed(&(dir.clone() + "status")).as_deref() == Some("connected");
        if !enabled && !connected {
            continue;
        }
        let label = std::fs::read(dir.clone() + "edid")
            .ok()
            .and_then(|edid| parse_edid_name(&edid))
            .unwrap_or_else(|| drm_connector_label(&name).to_string());
        if let Some(modes) = read_trimmed(&(dir + "modes")) {
            if let Some(first) = modes.lines().next() {
                if let Some((w, h)) = parse_mode_line(first) {
                    if w > 0 && h > 0 {
                        displays.push((label, w, h, 0));
                    }
                }
            }
        }
    }
    displays
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_displays() -> Vec<(String, u32, u32, u32)> {
    Vec::new()
}

#[cfg(target_os = "linux")]
#[repr(C)]
#[allow(dead_code)]
struct Statvfs {
    f_bsize: u64,
    f_frsize: u64,
    f_blocks: u64,
    f_bfree: u64,
    f_bavail: u64,
    f_files: u64,
    f_ffree: u64,
    f_favail: u64,
    f_fsid: u64,
    f_flag: u64,
    f_namemax: u64,
    spare: [i32; 6],
}

#[cfg(target_os = "linux")]
const _: () = assert!(std::mem::size_of::<Statvfs>() == 112);

#[cfg(target_os = "linux")]
unsafe extern "C" {
    fn statvfs(path: *const c_char, buf: *mut Statvfs) -> c_int;
}

#[cfg(target_os = "linux")]
fn decode_mount_escapes(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\\'
            && i + 4 <= chars.len()
            && chars[i + 1..i + 4].iter().all(|c| *c >= '0' && *c <= '7')
        {
            let code = (chars[i + 1] as u32 - '0' as u32) * 64
                + (chars[i + 2] as u32 - '0' as u32) * 8
                + (chars[i + 3] as u32 - '0' as u32);
            match char::from_u32(code) {
                Some(c) => out.push(c),
                None => {
                    out.push('\\');
                    out.extend(&chars[i + 1..i + 4]);
                }
            }
            i += 4;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

#[cfg(target_os = "linux")]
fn parse_mounts(content: &str) -> Vec<(String, String, String)> {
    let mut mounts = Vec::new();
    for line in content.lines() {
        let mut parts = line.split_whitespace();
        let (Some(device), Some(point), Some(fstype)) = (parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        mounts.push((
            decode_mount_escapes(device),
            decode_mount_escapes(point),
            fstype.to_string(),
        ));
    }
    mounts
}

#[cfg(target_os = "linux")]
fn keep_mountpoint(seen: &[String], device: &str, point: &str, fstype: &str) -> bool {
    if point == "/" {
        return true;
    }
    if device == "none" {
        return false;
    }
    if fstype == "zfs" {
        let pool = device.split('/').next().unwrap_or(device);
        if seen.iter().any(|seen| seen.split('/').next() == Some(pool)) {
            return false;
        }
        return true;
    }
    if fstype == "fuse.sshfs" || fstype == "bcachefs" {
        return true;
    }
    let dev = match device.strip_prefix("/dev/") {
        Some(dev) => dev,
        None => return false,
    };
    if dev.starts_with("loop") || dev.starts_with("ram") || dev.starts_with("fd") {
        return false;
    }
    if seen.iter().any(|seen| seen == device) {
        return false;
    }
    true
}

#[cfg(target_os = "linux")]
fn walk_mount_stats(mounts: &[(String, String, String)]) -> Vec<(String, u64, u64)> {
    let mut seen = Vec::new();
    let mut disks = Vec::new();
    for (device, point, fstype) in mounts {
        if !keep_mountpoint(&seen, device, point, fstype) {
            continue;
        }
        seen.push(device.clone());
        let path = match CString::new(point.as_str()) {
            Ok(path) => path,
            Err(_) => continue,
        };
        let mut stat: Statvfs = unsafe { std::mem::zeroed() };
        if unsafe { statvfs(path.as_ptr(), &mut stat) } != 0 {
            continue;
        }
        let total = stat.f_blocks.saturating_mul(stat.f_frsize);
        if total == 0 {
            continue;
        }
        let available = stat.f_bavail.saturating_mul(stat.f_frsize);
        disks.push((point.clone(), total.saturating_sub(available), total));
    }
    disks.sort();
    disks
}

#[cfg(target_os = "linux")]
fn get_disks() -> Vec<(String, u64, u64)> {
    match std::fs::read_to_string("/proc/mounts") {
        Ok(content) => walk_mount_stats(&parse_mounts(&content)),
        Err(_) => Vec::new(),
    }
}

#[cfg(target_os = "macos")]
fn get_cpu() -> String {
    let name = get_sysctl_str("machdep.cpu.brand_string").unwrap_or_default();
    if name.is_empty() {
        return "unknown".to_string();
    }
    let logical = get_sysctl_int("hw.logicalcpu_max")
        .or_else(|| get_sysctl_int("hw.ncpu"))
        .unwrap_or(0);
    if logical > 1 {
        format!("{name} ({logical} cores)")
    } else {
        name
    }
}

#[cfg(target_os = "linux")]
fn parse_cpuinfo(content: &str) -> (Option<String>, u32) {
    let mut name: Option<String> = None;
    let mut logical = 0u32;
    for line in content.lines() {
        if let Some((key, value)) = line.split_once(':') {
            if key.trim() == "processor" {
                logical += 1;
            } else if name.is_none() && (key.trim() == "model name" || key.trim() == "Hardware") {
                let value = value.trim().to_string();
                if !value.is_empty() {
                    name = Some(value);
                }
            }
        }
    }
    (name, logical)
}

#[cfg(target_os = "linux")]
fn get_cpu() -> String {
    let (mut name, mut logical) = std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .map(|content| parse_cpuinfo(&content))
        .unwrap_or((None, 0));
    if name.is_none() {
        name = std::fs::read_to_string("/sys/firmware/devicetree/base/model")
            .ok()
            .map(|content| {
                content
                    .trim_matches(|c: char| c.is_whitespace() || c == '\0')
                    .to_string()
            })
            .filter(|value| !value.is_empty());
    }
    if logical == 0 {
        logical = std::thread::available_parallelism()
            .map(|n| n.get() as u32)
            .unwrap_or(0);
    }
    match name {
        Some(name) if logical > 1 => format!("{name} ({logical} cores)"),
        Some(name) => name,
        None => "unknown".to_string(),
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_cpu() -> String {
    "unknown".to_string()
}

#[cfg(target_os = "macos")]
#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    fn IOServiceMatching(name: *const c_char) -> *mut std::os::raw::c_void;
    fn IOServiceGetMatchingServices(
        master: u32,
        matching: *mut std::os::raw::c_void,
        iterator: *mut u32,
    ) -> i32;
    fn IOIteratorNext(iterator: u32) -> u32;
    fn IORegistryEntryCreateCFProperties(
        entry: u32,
        props: *mut *mut std::os::raw::c_void,
        allocator: *mut std::os::raw::c_void,
        options: u32,
    ) -> i32;
    fn IORegistryEntryGetParentEntry(entry: u32, plane: *const c_char, parent: *mut u32) -> i32;
    fn IOObjectRelease(obj: u32) -> i32;
}

#[cfg(target_os = "macos")]
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFStringCreateWithCString(
        alloc: *mut std::os::raw::c_void,
        cstr: *const c_char,
        encoding: u32,
    ) -> *mut std::os::raw::c_void;
    fn CFDictionaryGetValue(
        dict: *mut std::os::raw::c_void,
        key: *const std::os::raw::c_void,
    ) -> *mut std::os::raw::c_void;
    fn CFStringGetCString(
        string: *mut std::os::raw::c_void,
        buffer: *mut u8,
        size: isize,
        encoding: u32,
    ) -> u8;
    fn CFNumberGetValue(number: *mut std::os::raw::c_void, num_type: i32, value: *mut i32) -> u8;
    fn CFGetTypeID(cf: *mut std::os::raw::c_void) -> u64;
    fn CFStringGetTypeID() -> u64;
    fn CFNumberGetTypeID() -> u64;
    fn CFRelease(cf: *mut std::os::raw::c_void);
}

#[cfg(target_os = "macos")]
fn cf_key(name: &[u8]) -> Option<*mut std::os::raw::c_void> {
    let key = unsafe {
        CFStringCreateWithCString(ptr::null_mut(), name.as_ptr() as *const c_char, 0x08000100)
    };
    if key.is_null() {
        return None;
    }
    Some(key)
}

#[cfg(target_os = "macos")]
fn cf_dict_string(dict: *mut std::os::raw::c_void, key: &[u8]) -> Option<String> {
    let key = cf_key(key)?;
    let value = unsafe { CFDictionaryGetValue(dict, key) };
    unsafe { CFRelease(key) };
    if value.is_null() {
        return None;
    }
    if unsafe { CFGetTypeID(value) } != unsafe { CFStringGetTypeID() } {
        return None;
    }
    let mut buf = [0u8; 256];
    if unsafe { CFStringGetCString(value, buf.as_mut_ptr(), buf.len() as isize, 0x08000100) } == 0 {
        return None;
    }
    let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    let text = std::str::from_utf8(&buf[..len]).ok()?;
    if text.is_empty() {
        return None;
    }
    Some(text.to_string())
}

#[cfg(target_os = "macos")]
fn cf_dict_i32(dict: *mut std::os::raw::c_void, key: &[u8]) -> Option<i32> {
    let key = cf_key(key)?;
    let value = unsafe { CFDictionaryGetValue(dict, key) };
    unsafe { CFRelease(key) };
    if value.is_null() {
        return None;
    }
    if unsafe { CFGetTypeID(value) } != unsafe { CFNumberGetTypeID() } {
        return None;
    }
    let mut out: i32 = 0;
    if unsafe { CFNumberGetValue(value, 3, &mut out) } == 0 {
        return None;
    }
    Some(out)
}

#[cfg(target_os = "macos")]
fn gpu_vendor_name(id: u32) -> &'static str {
    match id {
        0x106b => "Apple",
        0x8086 => "Intel",
        0x10de => "NVIDIA",
        0x1002 => "AMD",
        _ => "",
    }
}

#[cfg(target_os = "macos")]
fn gpu_name_from_props(props: *mut std::os::raw::c_void) -> Option<String> {
    cf_dict_string(props, b"model\0")
}

#[cfg(target_os = "macos")]
fn get_gpus() -> Vec<String> {
    let mut gpus = Vec::new();
    let matches = unsafe { IOServiceMatching(c"IOAccelerator".as_ptr()) };
    if matches.is_null() {
        return gpus;
    }
    let mut iterator = 0u32;
    if unsafe { IOServiceGetMatchingServices(0, matches, &mut iterator) } != 0 {
        unsafe { CFRelease(matches) };
        return gpus;
    }
    loop {
        let entry = unsafe { IOIteratorNext(iterator) };
        if entry == 0 {
            break;
        }
        let mut props: *mut std::os::raw::c_void = ptr::null_mut();
        if unsafe { IORegistryEntryCreateCFProperties(entry, &mut props, ptr::null_mut(), 0) } != 0
            || props.is_null()
        {
            unsafe { IOObjectRelease(entry) };
            continue;
        }
        let mut name = gpu_name_from_props(props);
        if name.is_none() {
            let mut parent = 0u32;
            if unsafe { IORegistryEntryGetParentEntry(entry, c"IOService".as_ptr(), &mut parent) }
                == 0
                && parent != 0
            {
                let mut parent_props: *mut std::os::raw::c_void = ptr::null_mut();
                if unsafe {
                    IORegistryEntryCreateCFProperties(parent, &mut parent_props, ptr::null_mut(), 0)
                } == 0
                    && !parent_props.is_null()
                {
                    name = gpu_name_from_props(parent_props);
                    unsafe { CFRelease(parent_props) };
                }
                unsafe { IOObjectRelease(parent) };
            }
        }
        let label = match name {
            Some(name) => name,
            None => match cf_dict_i32(props, b"vendor-id\0").map(|v| v as u32) {
                Some(id) => {
                    let vendor = gpu_vendor_name(id);
                    if vendor.is_empty() {
                        format!("GPU {id:04x}")
                    } else {
                        vendor.to_string()
                    }
                }
                None => "unknown".to_string(),
            },
        };
        unsafe { CFRelease(props) };
        unsafe { IOObjectRelease(entry) };
        gpus.push(label);
    }
    unsafe { IOObjectRelease(iterator) };
    gpus
}

#[cfg(target_os = "linux")]
fn parse_pci_modalias(modalias: &str) -> Option<(u16, u16, u8, u8)> {
    let rest = modalias.trim().strip_prefix("pci:")?;
    let rest = rest.strip_prefix('v')?;
    let vendor = u32::from_str_radix(rest.get(..8)?, 16).ok()? as u16;
    let rest = rest.get(8..)?.strip_prefix('d')?;
    let device = u32::from_str_radix(rest.get(..8)?, 16).ok()? as u16;
    let rest = rest.get(8..)?.strip_prefix("sv")?;
    let rest = rest.get(8..)?.strip_prefix("sd")?;
    let rest = rest.get(8..)?.strip_prefix("bc")?;
    let class = u8::from_str_radix(rest.get(..2)?, 16).ok()?;
    let rest = rest.get(2..)?.strip_prefix("sc")?;
    let subclass = u8::from_str_radix(rest.get(..2)?, 16).ok()?;
    Some((vendor, device, class, subclass))
}

#[cfg(target_os = "linux")]
fn lookup_pci_name(ids: &str, vendor: u16, device: u16) -> Option<String> {
    let vendor_key = format!("{vendor:04x}");
    let mut in_vendor = false;
    for line in ids.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if !line.starts_with(['\t', ' ']) {
            in_vendor = line.len() > 5
                && line[..4] == vendor_key
                && (line.as_bytes()[4] == b' ' || line.as_bytes()[4] == b'\t');
        } else if in_vendor && line.starts_with('\t') && !line.starts_with("\t\t") {
            if line.len() > 5 && line[1..5] == format!("{device:04x}") {
                let name = line[5..].trim().to_string();
                if !name.is_empty() {
                    return Some(name);
                }
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn load_pci_ids() -> Option<String> {
    for path in ["/usr/share/hwdata/pci.ids", "/usr/share/misc/pci.ids"] {
        if let Ok(content) = std::fs::read_to_string(path) {
            return Some(content);
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn linux_gpu_vendor_name(id: u16) -> &'static str {
    match id {
        0x8086 => "Intel",
        0x10de => "NVIDIA",
        0x1002 => "AMD",
        0x106b => "Apple",
        0x15ad => "VMware",
        0x1af4 => "Red Hat",
        0x80ee => "VirtualBox",
        _ => "",
    }
}

#[cfg(target_os = "linux")]
fn qualify_gpu_name(vendor: &str, device_name: &str) -> String {
    if device_name
        .to_lowercase()
        .starts_with(&vendor.to_lowercase())
    {
        device_name.to_string()
    } else {
        format!("{vendor} {device_name}")
    }
}

#[cfg(target_os = "linux")]
fn gpu_from_pci_dir(ids: Option<&str>, dir: &str, addr: &str) -> Option<(bool, String)> {
    let modalias = std::fs::read_to_string(dir.to_string() + "/modalias")
        .ok()
        .map(|content| content.trim().to_string())?;
    let (vendor, device, class, subclass) = parse_pci_modalias(&modalias)?;
    if class != 0x03 {
        return None;
    }
    let func = addr.rsplit('.').next()?.parse::<u32>().ok()?;
    if func > 0 && subclass == 0x80 {
        return None;
    }
    let name = ids
        .and_then(|ids| lookup_pci_name(ids, vendor, device))
        .map(|device_name| qualify_gpu_name(linux_gpu_vendor_name(vendor), &device_name))
        .unwrap_or_else(|| {
            let vendor_name = linux_gpu_vendor_name(vendor);
            if vendor_name.is_empty() {
                format!("GPU {vendor:04x}:{device:04x}")
            } else {
                format!("{vendor_name} [{vendor:04x}:{device:04x}]")
            }
        });
    let primary = std::fs::read_to_string(dir.to_string() + "/boot_vga")
        .is_ok_and(|content| content.trim() == "1");
    Some((primary, name))
}

#[cfg(target_os = "linux")]
fn walk_drm_gpus(ids: Option<&str>, base: &str) -> Vec<(bool, String)> {
    let mut gpus = Vec::new();
    let Ok(entries) = std::fs::read_dir(base) else {
        return gpus;
    };
    let mut names: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.starts_with("card") && !name.contains('-'))
        .collect();
    names.sort();
    for name in names {
        let dir = base.to_string() + "/" + &name + "/device";
        let addr = std::fs::read_link(dir.clone())
            .ok()
            .and_then(|target| {
                target
                    .file_name()
                    .and_then(|base| base.to_str())
                    .map(|base| base.to_string())
            })
            .unwrap_or_default();
        if let Some(gpu) = gpu_from_pci_dir(ids, &dir, &addr) {
            gpus.push(gpu);
        }
    }
    gpus
}

#[cfg(target_os = "linux")]
fn walk_pci_gpus(ids: Option<&str>, base: &str) -> Vec<(bool, String)> {
    let mut gpus = Vec::new();
    let Ok(entries) = std::fs::read_dir(base) else {
        return gpus;
    };
    let mut names: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    for name in names {
        let dir = base.to_string() + "/" + &name;
        if let Some(gpu) = gpu_from_pci_dir(ids, &dir, &name) {
            gpus.push(gpu);
        }
    }
    gpus
}

#[cfg(target_os = "linux")]
fn get_gpus_from(drm_base: &str, pci_base: &str, ids: Option<&str>) -> Vec<String> {
    let mut gpus = walk_drm_gpus(ids, drm_base);
    if gpus.is_empty() {
        gpus = walk_pci_gpus(ids, pci_base);
    }
    gpus.sort_by_key(|gpu| !gpu.0);
    gpus.into_iter().map(|(_, name)| name).collect()
}

#[cfg(target_os = "linux")]
fn get_gpus() -> Vec<String> {
    let ids = load_pci_ids();
    get_gpus_from("/sys/class/drm", "/sys/bus/pci/devices", ids.as_deref())
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_gpus() -> Vec<String> {
    Vec::new()
}

fn format_bytes(used: u64, total: u64) -> String {
    if total == 0 {
        return "unknown".to_string();
    }
    let (divisor, unit) = if total >= 1 << 40 {
        (1u64 << 40, "TiB")
    } else if total >= 1 << 30 {
        (1u64 << 30, "GiB")
    } else if total >= 1 << 20 {
        (1u64 << 20, "MiB")
    } else if total >= 1 << 10 {
        (1u64 << 10, "KiB")
    } else {
        (1, "B")
    };
    let percent = (used as u128 * 100 / total as u128) as u64;
    format!(
        "{:.2} {unit} / {:.2} {unit} ({percent}%)",
        used as f64 / divisor as f64,
        total as f64 / divisor as f64
    )
}

#[cfg(target_os = "macos")]
#[repr(C)]
#[allow(dead_code)]
struct VmStatistics64 {
    free_count: u32,
    active_count: u32,
    inactive_count: u32,
    wire_count: u32,
    zero_fill_count: u64,
    reactivations: u64,
    pageins: u64,
    pageouts: u64,
    faults: u64,
    cow_faults: u64,
    lookups: u64,
    hits: u64,
    purges: u64,
    purgeable_count: u32,
    speculative_count: u32,
    decompressions: u64,
    compressions: u64,
    swapins: u64,
    swapouts: u64,
    compressor_page_count: u32,
    throttled_count: u32,
    external_page_count: u32,
    internal_page_count: u32,
    total_uncompressed_pages_in_compressor: u64,
    swapped_count: u64,
    total_tag_storage_pages: u64,
    nontag_pageable_tag_storage_pages: u64,
    nontag_wired_tag_storage_pages: u64,
    free_tag_storage_pages: u64,
    tag_storing_tag_storage_pages: u64,
    total_tagged_pages: u64,
    resident_tagged_pages: u64,
    compressed_tagged_pages: u64,
    tagged_compressions: u64,
    tagged_decompressions: u64,
    compressed_tag_storage_bytes: u64,
    speculative_pages_created: u64,
    speculative_pages_activated: u64,
}

#[cfg(target_os = "macos")]
const _: () = assert!(std::mem::size_of::<VmStatistics64>() == 264);

#[cfg(target_os = "macos")]
fn get_ram() -> String {
    unsafe extern "C" {
        fn mach_host_self() -> u32;
        fn host_statistics64(
            host: u32,
            flavor: i32,
            info: *mut VmStatistics64,
            count: *mut u32,
        ) -> i32;
    }
    let total = match get_sysctl_u64("hw.memsize") {
        Some(total) if total > 0 => total,
        _ => return "unknown".to_string(),
    };
    let page_size = match get_sysctl_int("hw.pagesize") {
        Some(size) if size > 0 => size as u64,
        _ => return "unknown".to_string(),
    };
    let mut stats: VmStatistics64 = unsafe { std::mem::zeroed() };
    let mut count = (std::mem::size_of::<VmStatistics64>() / 4) as u32;
    let host = unsafe { mach_host_self() };
    if unsafe { host_statistics64(host, 4, &mut stats, &mut count) } != 0 {
        return "unknown".to_string();
    }
    let free_pages = stats.free_count.saturating_sub(stats.speculative_count) as u64;
    let available = (free_pages + stats.external_page_count as u64).saturating_mul(page_size);
    format_bytes(total.saturating_sub(available), total)
}

#[cfg(target_os = "linux")]
fn parse_meminfo_value(content: &str, key: &str) -> u64 {
    for line in content.lines() {
        if let Some((name, value)) = line.split_once(':') {
            if name.trim() == key {
                if let Some(number) = value.split_whitespace().next() {
                    if let Ok(kb) = number.parse::<u64>() {
                        return kb;
                    }
                }
            }
        }
    }
    0
}

#[cfg(target_os = "linux")]
fn get_ram() -> String {
    let content = match std::fs::read_to_string("/proc/meminfo") {
        Ok(content) => content,
        Err(_) => return "unknown".to_string(),
    };
    let total = parse_meminfo_value(&content, "MemTotal");
    if total == 0 {
        return "unknown".to_string();
    }
    let mut available = parse_meminfo_value(&content, "MemAvailable");
    if available == 0 || available >= total {
        available = parse_meminfo_value(&content, "MemFree")
            .saturating_add(parse_meminfo_value(&content, "Buffers"))
            .saturating_add(parse_meminfo_value(&content, "Cached"))
            .saturating_add(parse_meminfo_value(&content, "SReclaimable"))
            .saturating_sub(parse_meminfo_value(&content, "Shmem"));
    }
    let used = total.saturating_sub(available);
    format_bytes(used.saturating_mul(1024), total.saturating_mul(1024))
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_ram() -> String {
    "unknown".to_string()
}

#[cfg(target_os = "macos")]
#[repr(C)]
#[allow(dead_code)]
struct Statfs {
    f_bsize: u32,
    f_iosize: i32,
    f_blocks: u64,
    f_bfree: u64,
    f_bavail: u64,
    f_files: u64,
    f_ffree: u64,
    f_fsid: [i32; 2],
    f_owner: u32,
    f_type: u32,
    f_flags: u32,
    f_fssubtype: u32,
    f_fstypename: [u8; 16],
    f_mntonname: [u8; 1024],
    f_mntfromname: [u8; 1024],
    f_flags_ext: u32,
    f_reserved: [u32; 7],
}

#[cfg(target_os = "macos")]
const _: () = assert!(std::mem::size_of::<Statfs>() == 2168);

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn getfsstat(buf: *mut Statfs, bufsize: i32, flags: i32) -> i32;
}

#[cfg(target_os = "macos")]
fn statfs_str(bytes: &[u8]) -> String {
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    std::str::from_utf8(&bytes[..len])
        .unwrap_or_default()
        .to_string()
}

#[cfg(target_os = "macos")]
fn get_disks() -> Vec<(String, u64, u64)> {
    let mut disks = Vec::new();
    let count = unsafe { getfsstat(ptr::null_mut(), 0, 2) };
    if count <= 0 {
        return disks;
    }
    let mut buf: Vec<Statfs> = (0..count).map(|_| unsafe { std::mem::zeroed() }).collect();
    let bytes = count as usize * std::mem::size_of::<Statfs>();
    let fetched = unsafe { getfsstat(buf.as_mut_ptr(), bytes as i32, 2) };
    if fetched <= 0 {
        return disks;
    }
    for fs in buf.iter().take(fetched as usize) {
        if (fs.f_flags & 0x00100000) != 0 {
            continue;
        }
        let point = statfs_str(&fs.f_mntonname);
        if point.is_empty() {
            continue;
        }
        let from = statfs_str(&fs.f_mntfromname);
        if point != "/" && !from.starts_with("/dev/") {
            continue;
        }
        let total = fs.f_blocks.saturating_mul(fs.f_bsize as u64);
        if total == 0 {
            continue;
        }
        let available = fs.f_bavail.saturating_mul(fs.f_bsize as u64);
        disks.push((point, total.saturating_sub(available), total));
    }
    disks.sort();
    disks
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_disks() -> Vec<(String, u64, u64)> {
    Vec::new()
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
const IFF_UP: u32 = 0x1;
#[cfg(any(target_os = "linux", target_os = "macos"))]
const IFF_LOOPBACK: u32 = 0x8;
#[cfg(any(target_os = "linux", target_os = "macos"))]
const AF_INET_VALUE: u16 = 2;

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[repr(C)]
struct IfAddrs {
    next: *mut IfAddrs,
    name: *const c_char,
    flags: u32,
    addr: *mut std::os::raw::c_void,
    netmask: *mut std::os::raw::c_void,
    dst: *mut std::os::raw::c_void,
    data: *mut std::os::raw::c_void,
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
unsafe extern "C" {
    fn getifaddrs(list: *mut *mut IfAddrs) -> c_int;
    fn freeifaddrs(list: *mut IfAddrs);
}

#[cfg(target_os = "linux")]
fn sockaddr_family(addr: *mut std::os::raw::c_void) -> u16 {
    unsafe { *(addr as *const u16) }
}

#[cfg(target_os = "macos")]
fn sockaddr_family(addr: *mut std::os::raw::c_void) -> u16 {
    unsafe { *((addr as *const u8).add(1)) as u16 }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn sockaddr_ipv4(addr: *mut std::os::raw::c_void) -> Option<[u8; 4]> {
    if addr.is_null() || sockaddr_family(addr) != AF_INET_VALUE {
        return None;
    }
    let mut octets = [0u8; 4];
    unsafe { std::ptr::copy_nonoverlapping((addr as *const u8).add(4), octets.as_mut_ptr(), 4) };
    Some(octets)
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn prefix_len(mask: &[u8; 4]) -> u8 {
    mask.iter().map(|b| b.count_ones() as u8).sum()
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn list_ipv4_interfaces() -> Vec<(String, [u8; 4], u8)> {
    let mut interfaces = Vec::new();
    let mut list: *mut IfAddrs = std::ptr::null_mut();
    if unsafe { getifaddrs(&mut list) } != 0 || list.is_null() {
        return interfaces;
    }
    let mut current = list;
    while !current.is_null() {
        let entry = unsafe { &*current };
        current = entry.next;
        if entry.name.is_null() {
            continue;
        }
        if entry.flags & IFF_UP == 0 || entry.flags & IFF_LOOPBACK != 0 {
            continue;
        }
        let ip = match sockaddr_ipv4(entry.addr) {
            Some(ip) => ip,
            None => continue,
        };
        if ip[0] == 127 {
            continue;
        }
        let mask = match sockaddr_ipv4(entry.netmask) {
            Some(mask) => mask,
            None => continue,
        };
        let name = match unsafe { std::ffi::CStr::from_ptr(entry.name) }.to_str() {
            Ok(name) if !name.is_empty() => name.to_string(),
            _ => continue,
        };
        interfaces.push((name, ip, prefix_len(&mask)));
    }
    unsafe { freeifaddrs(list) };
    interfaces
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn default_route_ip() -> Option<[u8; 4]> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("1.1.1.1:80").ok()?;
    match socket.local_addr().ok()? {
        std::net::SocketAddr::V4(addr) => Some(addr.ip().octets()),
        _ => None,
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn pick_local_ip(
    interfaces: &[(String, [u8; 4], u8)],
    default: Option<[u8; 4]>,
) -> Option<(String, [u8; 4], u8)> {
    if let Some(ip) = default
        && let Some(found) = interfaces.iter().find(|iface| iface.1 == ip)
    {
        return Some(found.clone());
    }
    interfaces.first().cloned()
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn format_local_ip(name: &str, ip: &[u8; 4], prefix: u8) -> String {
    format!(
        "({name}): {}.{}.{}.{}/{}",
        ip[0], ip[1], ip[2], ip[3], prefix
    )
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn get_local_ip() -> Option<String> {
    let interfaces = list_ipv4_interfaces();
    if interfaces.is_empty() {
        return None;
    }
    let (name, ip, prefix) = pick_local_ip(&interfaces, default_route_ip())?;
    Some(format_local_ip(&name, &ip, prefix))
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_local_ip() -> Option<String> {
    None
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
unsafe extern "C" {
    fn isatty(fd: i32) -> i32;
}

fn colors_enabled() -> bool {
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        unsafe { isatty(1) == 1 }
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        false
    }
}

fn char_width(c: char) -> usize {
    match c as u32 {
        0x0300..=0x036F | 0x200D | 0xFE00..=0xFE0F => 0,
        0x1100..=0x115F
        | 0x2E80..=0xA4CF
        | 0xAC00..=0xD7A3
        | 0xF900..=0xFAFF
        | 0xFE30..=0xFE4F
        | 0xFF00..=0xFF60
        | 0xFFE0..=0xFFE6
        | 0x2600..=0x27BF
        | 0x2B00..=0x2BFF
        | 0x1F300..=0x1FAFF
        | 0x20000..=0x3FFFD => 2,
        _ => 1,
    }
}

fn logo_visible_width(line: &str) -> usize {
    let mut width = 0;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '$' {
            match chars.peek() {
                Some('1'..='9') => {
                    chars.next();
                }
                Some('$') => {
                    chars.next();
                    width += 1;
                }
                _ => {
                    width += 1;
                }
            }
        } else {
            width += char_width(c);
        }
    }
    width
}

fn paint_logo_line(
    line: &str,
    colors: &[&str],
    carry: &mut String,
    colorize: bool,
    out: &mut String,
) {
    if logo_visible_width(line) == 0 {
        return;
    }
    if colorize {
        out.push_str("\x1b[1m");
        out.push_str(carry);
    }
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '$' {
            match chars.peek() {
                Some(digit) if ('1'..='9').contains(digit) => {
                    let index = *digit as usize - '1' as usize;
                    chars.next();
                    if colorize {
                        let code = colors.get(index).copied().unwrap_or("\x1b[m");
                        out.push_str(code);
                        carry.clear();
                        carry.push_str(code);
                    }
                }
                Some('$') => {
                    chars.next();
                    out.push('$');
                }
                _ => {
                    out.push('$');
                }
            }
        } else {
            out.push(c);
        }
    }
    if colorize {
        out.push_str("\x1b[m");
    }
}

fn select_logo(candidates: &[&str]) -> &'static LogoEntry {
    for candidate in candidates {
        if candidate.is_empty() || !candidate.as_bytes()[0].is_ascii_alphabetic() {
            continue;
        }
        for entry in LOGOS {
            if entry
                .names
                .iter()
                .any(|name| name.eq_ignore_ascii_case(candidate))
            {
                return entry;
            }
        }
    }
    &LOGO_UNKNOWN
}

#[cfg(target_os = "macos")]
fn select_platform_logo() -> &'static LogoEntry {
    select_logo(&["apple"])
}

#[cfg(target_os = "linux")]
fn select_platform_logo() -> &'static LogoEntry {
    let mut candidates: Vec<String> = Vec::new();
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if let Some(id) = line.strip_prefix("ID=") {
                let id = id.trim_matches('"').trim().to_string();
                if !id.is_empty() {
                    candidates.push(id);
                }
            } else if let Some(like) = line.strip_prefix("ID_LIKE=") {
                for token in like.trim_matches('"').split_whitespace() {
                    if !token.is_empty() {
                        candidates.push(token.to_string());
                    }
                }
            }
        }
    }
    candidates.push("linux".to_string());
    let refs: Vec<&str> = candidates.iter().map(|s| s.as_str()).collect();
    select_logo(&refs)
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn select_platform_logo() -> &'static LogoEntry {
    static EMPTY: LogoEntry = LogoEntry {
        names: &[],
        colors: &[],
        art: &[],
    };
    &EMPTY
}

fn main() {
    let mut info = Vec::new();
    info.push(format!("Hostname: {}", get_hostname()));
    info.push(format!("OS: {}", get_os_info()));
    info.push(format!("Kernel: {}", get_kernel()));
    info.push(format!("Device: {}", get_device()));
    info.push(format!("Uptime: {}", get_uptime()));
    let shell = get_shell();
    info.push(format!("Shell: {shell}"));
    info.push(format!("Init: {}", get_init()));
    info.push(format!("Terminal: {}", get_terminal(&shell)));
    info.push(format!("CPU: {}", get_cpu()));
    info.push(format!("RAM: {}", get_ram()));
    let gpus = get_gpus();
    if gpus.is_empty() {
        info.push("GPU: unknown".to_string());
    } else if gpus.len() == 1 {
        info.push(format!("GPU: {}", gpus[0]));
    } else {
        for (i, gpu) in gpus.iter().enumerate() {
            info.push(format!("GPU ({}): {}", i + 1, gpu));
        }
    }
    let displays = get_displays();
    if displays.is_empty() {
        info.push("Display: unknown".to_string());
    } else if displays.len() == 1 {
        let (_, w, h, hz) = &displays[0];
        info.push(format!("Display: {}", format_resolution(*w, *h, *hz)));
    } else {
        for (i, (label, w, h, hz)) in displays.iter().enumerate() {
            let tag = if label.is_empty() {
                (i + 1).to_string()
            } else {
                label.clone()
            };
            info.push(format!(
                "Display ({}): {}",
                tag,
                format_resolution(*w, *h, *hz)
            ));
        }
    }
    let disks = get_disks();
    if disks.is_empty() {
        info.push("Disk: unknown".to_string());
    } else {
        for (point, used, total) in &disks {
            info.push(format!("Disk ({}): {}", point, format_bytes(*used, *total)));
        }
    }
    match get_local_ip() {
        Some(line) => info.push(format!("Local IP {line}")),
        None => info.push("Local IP: unknown".to_string()),
    }
    let logo = select_platform_logo();
    let colorize = colors_enabled();
    let mut carry = String::new();
    if colorize && let Some(base) = logo.colors.first() {
        carry.push_str(base);
    }
    let width = logo
        .art
        .iter()
        .map(|line| logo_visible_width(line))
        .max()
        .unwrap_or(0);
    let rows = logo.art.len().max(info.len());
    for i in 0..rows {
        match (logo.art.get(i), info.get(i)) {
            (Some(art), Some(text)) => {
                let mut line = String::new();
                paint_logo_line(art, logo.colors, &mut carry, colorize, &mut line);
                let pad = width.saturating_sub(logo_visible_width(art));
                for _ in 0..pad {
                    line.push(' ');
                }
                line.push_str("  ");
                line.push_str(text);
                println!("{line}");
            }
            (Some(art), None) => {
                let mut line = String::new();
                paint_logo_line(art, logo.colors, &mut carry, colorize, &mut line);
                println!("{line}");
            }
            (None, Some(text)) => {
                if width == 0 {
                    println!("{text}");
                } else {
                    println!("{:width$}  {text}", "", width = width);
                }
            }
            (None, None) => {}
        }
    }
}
