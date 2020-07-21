use std::collections::HashMap;
use std::env::{var, var_os};
use std::ffi::{OsStr, OsString};
use std::fs::{File, Permissions, remove_dir_all, remove_file, set_permissions, symlink_metadata};
use std::io::{Error, Write};
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::ptr::null;
use std::thread;
use std::time::Duration;
use std::usize::MAX as MAX_DEPTH;

use chrono::{DateTime, Local};
use pretty_bytes::converter::convert;
use walkdir::WalkDir;
use winapi::um::winbase::{MOVEFILE_DELAY_UNTIL_REBOOT, MoveFileExW};

use crate::globals::{KPTEMP_VERSION, LABEL_HANDLE, PROGRESS_HANDLE, TOTAL_STEP};
use crate::gui::progress_bar::advance_progress_bar;
use crate::gui::windows_helper::set_window_text;
use crate::Ignore;
use crate::privilege::adjust_privilege;
use crate::utils::{data_recycle_bin, empty_recycle_bin, error_box, message_box, restart};

// TODO Please Refactor ME

#[derive(PartialEq, Eq)]
enum DeletionType {
    Clear,
    Extension,
}

struct TempComponent {
    path: PathBuf,
    size: u64,
    len: u64,
    depth: usize,
    min_depth: usize,
    extension: &'static OsStr,
    deletion_type: DeletionType,
    need_reboot: bool,
}

impl TempComponent {
    fn new_clear(path: PathBuf) -> TempComponent {
        TempComponent {
            path,
            size: 0,
            len: 0,
            extension: "".as_ref(),
            depth: MAX_DEPTH,
            min_depth: 1,
            deletion_type: DeletionType::Clear,
            need_reboot: false,
        }
    }

    fn new_extension(path: PathBuf, extension: &'static OsStr) -> TempComponent {
        TempComponent {
            path,
            size: 0,
            len: 0,
            extension,
            depth: 1,
            min_depth: 1,
            deletion_type: DeletionType::Extension,
            need_reboot: false,
        }
    }
}

fn remove_readonly(path: &Path) {
    match symlink_metadata(path) {
        Ok(metadata) => {
            let mut permissions: Permissions = metadata.permissions();
            permissions.set_readonly(false);
            set_permissions(path, permissions).ignore();
        }
        _ => {}
    }
}

fn file_has_extension(path: &Path, ext_to_search: &OsStr) -> bool {
    if path.is_file() {
        return match path.extension() {
            Some(e) => e == ext_to_search,
            None => false,
        };
    }

    return false;
}

fn clear(path: &Path, component: &mut TempComponent) {
    if component.deletion_type == DeletionType::Extension
        && file_has_extension(&path, &component.extension) == false {
        return;
    }

    remove_readonly(path);

    if path.is_dir() {
        unsafe { set_window_text(LABEL_HANDLE, &format!("Remove {} ...", path.display())); }
        let _ = remove_dir_all(path);
    } else if path.is_file() {
        let _ = remove_file(path);
    }
}

fn get_len_and_size(path: &Path, component: &mut TempComponent) {
    if path.is_dir() {
        unsafe { set_window_text(LABEL_HANDLE, &format!("Indexing {} ...", path.display())); }
    }

    if path.is_file() == false {
        return;
    }

    if component.deletion_type == DeletionType::Extension
        && file_has_extension(&path, &component.extension) == false {
        return;
    }

    component.len += 1;
    component.size += match path.metadata() {
        Ok(metadata) => metadata.len(),
        _ => 0,
    };
}

fn remove_on_reboot(path: &Path, component: &mut TempComponent) {
    if component.deletion_type == DeletionType::Extension
        && file_has_extension(&path, &component.extension) == false {
        return;
    }

    if !path.is_file() && !path.is_dir() {
        return;
    }

    if path.is_dir() {
        unsafe { set_window_text(LABEL_HANDLE, &format!("Remove on reboot {} ...", path.display())); }
    }

    remove_readonly(path);
    let existing_filename: Vec<u16> = path.as_os_str().encode_wide().collect();

    let mut r_at_reboot = || {
        let status: i32 = unsafe {
            MoveFileExW(
                existing_filename.as_ptr(),
                null(),
                MOVEFILE_DELAY_UNTIL_REBOOT,
            )
        };

        if status != 0 {
            component.need_reboot = true;
        }
    };

    if path.is_dir() {
        match remove_dir_all(path) {
            Ok(_) => {}
            _ => r_at_reboot()
        }
    } else if path.is_file() {
        match remove_file(path) {
            Ok(_) => {}
            _ => r_at_reboot()
        }
    }
}

fn walk<F>(component: &mut TempComponent, callback: F)
    where F: Fn(&Path, &mut TempComponent) {
    let walk = match component.min_depth {
        0 => {
            WalkDir::new(component.path.as_path())
                .follow_links(false)
                .max_depth(component.depth)
        }
        _ => {
            WalkDir::new(component.path.as_path())
                .follow_links(false)
                .min_depth(component.min_depth)
                .max_depth(component.depth)
        }
    };

    for entry in walk.into_iter().filter_map(|e| e.ok()) {
        let path: &Path = entry.path();
        callback(path, component);
    }

    advance_progress_bar(unsafe { PROGRESS_HANDLE }, 1);
}

fn get_walk_glob_file(
    temp_components: &mut Vec<TempComponent>,
    system_vars: &HashMap<&'static str, PathBuf>) -> () {
    let config = [
        ["system_drive", "tmp"],
        ["system_root", "tmp"],
        ["sys32", "tmp"],
    ];

    for row in config.iter() {
        let key: &str = row[0];
        let ext: &OsStr = OsStr::new(row[1]);
        let path: &PathBuf = system_vars.get(key).unwrap();

        if !(path.exists() && path.is_dir()) {
            continue;
        }

        let mut temp_component = TempComponent::new_extension(path.to_path_buf(), ext);
        walk(&mut temp_component, get_len_and_size);

        if temp_component.len > 1 {
            temp_components.push(temp_component);
        }
    }
}

fn get_glob_folders_clear(
    temp_components: &mut Vec<TempComponent>,
    users_profile_dirs: &Vec<PathBuf>) -> () {
    let list = [
        ["firefox_profil", "AppData\\Local\\Mozilla\\Firefox\\Profiles"],
        ["waterfox_profil", "AppData\\Local\\Mozilla\\Waterfox\\Profiles"],
        ["seamonkey_profil", "AppData\\Local\\Mozilla\\SeaMonkey\\Profiles"],
        ["palemoon_profil", "AppData\\Local\\Mozilla\\Pale Moon\\Profiles"],
        ["icedragon_profil", "AppData\\Local\\Comodo\\IceDragon\\Profiles"],
    ];

    let mut profiles: HashMap<String, Vec<PathBuf>> = HashMap::new();

    for l in list.iter() {
        let k = l[0].to_string();
        let format_path = PathBuf::from(l[1]);
        let mut v = Vec::new();

        for user_dir in users_profile_dirs {
            let path: PathBuf = user_dir.join(&format_path);

            if path.exists() && path.is_dir() {
                for entry in WalkDir::new(path.as_path())
                    .follow_links(false)
                    .min_depth(1)
                    .max_depth(1)
                    .into_iter()
                    .filter_map(|e| e.ok()) {
                    let nested: PathBuf = entry.into_path();

                    if nested.is_dir() {
                        v.push(nested);
                    }
                }
            }
        }

        profiles.insert(k, v);
    }

    let config = [
        ["firefox_profil", "cache2"],
        ["firefox_profil", "OfflineCache"],
        ["firefox_profil", "jumpListCache"],
        ["firefox_profil", "startupCache"],
        ["waterfox_profil", "cache2"],
        ["waterfox_profil", "OfflineCache"],
        ["waterfox_profil", "jumpListCache"],
        ["waterfox_profil", "startupCache"],
        ["seamonkey_profil", ""],
        ["palemoon_profil", ""],
        ["icedragon_profil", "Cache"],
        ["icedragon_profil", "OfflineCache"],
        ["icedragon_profil", "mozilla-media-cache"],
        ["icedragon_profil", "startupCache"],
        ["icedragon_profil", "jumpListCache"],
    ];

    for c in config.iter() {
        let var: String = c[0].to_string();
        let format_path: PathBuf = PathBuf::from(c[1]);
        match profiles.get(&var) {
            Some(list) => {
                for p in list.iter() {
                    let path: PathBuf = p.join(&format_path);

                    if check_path(&path, false, false) {
                        let mut temp_component = TempComponent::new_clear(path);
                        walk(&mut temp_component, get_len_and_size);

                        if temp_component.len > 1 {
                            temp_components.push(temp_component);
                        }
                    }
                }
            }
            _ => continue,
        };
    }
}

fn get_walk_all(
    temp_components: &mut Vec<TempComponent>,
    system_vars: &HashMap<&'static str, PathBuf>,
    users_profile_dirs: &Vec<PathBuf>,
) -> () {
    let config = [
        ["system_root", "Temp"],
        ["system_root", "Prefetch"],
        ["system_root", "SoftwareDistribution\\Download"],
        ["system_root", "SoftwareDistribution\\DataStore\\Logs"],
        ["system_root", "SoftwareDistribution\\DataStore.bak\\Logs"],
        ["system_root", "Logs\\waasmedic"],
        ["system_root", "Logs\\WindowsUpdate"],
        ["all_user_profile", "Temp"],
        ["users", "AppData\\Local\\Temp"],
        ["users", "AppData\\Local\\Microsoft\\Windows\\Temporary Internet Files"],
        ["users", "AppData\\Roaming\\Macromedia\\Flash Player\\#SharedObjects"],
        ["users", "AppData\\Local\\Microsoft\\Windows\\WebCache"],
        // ["users", "AppData\\Local\\Microsoft\\Windows\\AppCache"],
        // ["users", "AppData\\Local\\Microsoft\\Windows\\Caches"],
        ["users", "AppData\\Local\\Microsoft\\Windows\\INetCache\\IE"],
        ["system_root", "ie7updates"],
        ["system_root", "ie8updates"],
        ["users", "AppData\\Local\\Google\\Chrome\\User Data\\Default\\Cache"],
        ["users", "AppData\\Local\\Google\\Chrome\\User Data\\Default\\File System"],
        ["users", "AppData\\Local\\Google\\Chrome\\UserData\\Default\\Local Storage"],
        ["users", "AppData\\Local\\BraveSoftware\\Brave-Browser\\User Data\\Default\\Cache"],
        ["users", "AppData\\LocalLow\\Sun\\Java\\Deployment\\cache"],
        ["users", "AppData\\Local\\Opera Software\\Opera Stable\\Cache"],
        ["users", "AppData\\Local\\Packages\\Microsoft.MicrosoftEdge_8wekyb3d8bbwe\\AC\\MicrosoftEdge\\Cache"],
        ["users", "AppData\\Local\\Yandex\\YandexBrowser\\Default\\Cache"],
        ["users", "AppData\\Local\\Yandex\\YandexBrowser\\User DataDefault\\Cache"],
        ["users", "AppData\\Local\\Chromium\\User Data\\Default\\Cache"],
        ["users", "AppData\\Local\\Chromium\\User Data\\Default\\ApplicationCache"],
        ["users", "AppData\\Local\\Comodo\\Dragon\\User Data\\Default\\Cache"],
        ["users", "AppData\\Local\\Comodo\\Dragon\\User DataDefault\\Cache"],
        ["users", "AppData\\Local\\Baidu\\User Data\\Cache"],
        ["users", "AppData\\Local\\Vivaldi\\User Data\\Default\\Application Cache"],
        ["users", "AppData\\Local\\Vivaldi\\User Data\\Default\\Cache"],
        ["users", "AppData\\Local\\Vivaldi\\User Data\\Default\\GPUCache"],
        ["users", "AppData\\Local\\Vivaldi\\User Data\\Default\\Media Cache"],
        ["system_drive", "\\Config.Msi"],
    ];

    for row in config.iter() {
        let var: &str = row[0];
        let format_path: PathBuf = PathBuf::from(row[1]);

        let mut build_component = |path_of_component: PathBuf| -> () {
            if check_path(&path_of_component, false, false) {
                let mut temp_component = TempComponent::new_clear(path_of_component);
                walk(&mut temp_component, get_len_and_size);

                if temp_component.len > 1 {
                    temp_components.push(temp_component);
                }
            }
        };

        if var == "users" {
            for user_dir in users_profile_dirs {
                let path = user_dir.join(&format_path);

                build_component(path);
            }
        } else {
            let sys_path: &PathBuf = system_vars.get(var).unwrap();
            let path = sys_path.join(&format_path);

            build_component(path);
        }
    }

    let user_temp = match var("USERPROFILE") {
        Ok(p) => Some(format!("{}\\AppData\\Local\\Temp", p)),
        _ => None
    };

    let mut temp = var("TEMP").ok();

    if user_temp.is_none() || temp.is_none() {
        return;
    }

    if temp == user_temp {
        return;
    }

    let temp = temp.get_or_insert("".to_string()).as_str();

    if temp == "" {
        return;
    }

    let path = PathBuf::from(temp);

    if check_path(&path, true, false) {
        let mut temp_component = TempComponent::new_clear(path);
        walk(&mut temp_component, get_len_and_size);

        if temp_component.len > 1 {
            temp_components.push(temp_component);
        }
    }

    advance_progress_bar(unsafe { PROGRESS_HANDLE }, 1);
}

fn windows_old(
    temp_components: &mut Vec<TempComponent>,
    system_vars: &HashMap<&'static str, PathBuf>,
) {
    let system_drive = system_vars.get("system_drive").unwrap();

    let paths = [
        PathBuf::from("\\Windows.old"),
        PathBuf::from("\\Windows.old.000"),
        PathBuf::from("\\Windows.old.001"),
        PathBuf::from("\\Windows.old.002"),
        PathBuf::from("\\Windows.old.003"),
        PathBuf::from("\\Windows.old.004"),
        PathBuf::from("\\Windows.old.005"),
        PathBuf::from("\\$WINDOWS.~BT"),
        PathBuf::from("\\$Windows.~WS"),
    ];

    for p in paths.iter() {
        let path = &system_drive.clone().join(p);

        if check_path(path, false, false) {
            let mut temp_component = TempComponent::new_clear(path.clone());

            walk(&mut temp_component, get_len_and_size);

            if temp_component.len > 1 {
                temp_component.min_depth = 0;
                temp_components.push(temp_component);
            }
        }
    }
}

fn get_users_dirs(system_vars: &HashMap<&'static str, PathBuf>) -> Vec<PathBuf> {
    let mut users_dirs: Vec<PathBuf> = Vec::new();
    let base_user_profile: &PathBuf = system_vars.get("base_user_profile").unwrap();
    let system_root: &PathBuf = system_vars.get("system_root").unwrap();

    for entry in WalkDir::new(&base_user_profile)
        .follow_links(false)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok()) {
        let p: &Path = entry.path();

        if p.is_dir() {
            users_dirs.push(p.to_path_buf());
        }
    }

    let local_service_profile: PathBuf = system_root.join(PathBuf::from("ServiceProfiles\\LocalService"));
    let network_service_profile: PathBuf = system_root.join(PathBuf::from("NetworkService\\LocalService"));

    users_dirs.push(local_service_profile);
    users_dirs.push(network_service_profile);

    users_dirs
}

fn check_path(path: &PathBuf, check_temp: bool, panic: bool) -> bool {
    if !path.is_absolute() || !path.has_root() || !path.exists() || !path.is_dir() {
        if panic == true {
            error_box(format!("Invalid Path {} ... Abandoned for Safety", path.display()).to_string());
            restart();
            panic!("Err");
        }

        return false;
    }

    if check_temp == true {
        match path.file_name() {
            Some(f) => {
                if f != "Temp" && f != "Tmp" && f != "temp" && f != "tmp" {
                    if panic == true {
                        error_box(format!("Invalid Path {} ... Abandoned for Safety", path.display()).to_string());
                        restart();
                        panic!("Err");
                    }

                    return false;
                }
            }
            _ => {
                if panic == true {
                    error_box(format!("Invalid Path {} ... Abandoned for Safety", path.display()).to_string());
                    restart();
                    panic!("Err");
                }

                return false;
            }
        }
    }

    return true;
}

fn get_system_vars() -> HashMap<&'static str, PathBuf> {
    let mut system_vars = HashMap::new();

    let system_drive_string: String = var("SYSTEMDRIVE")
        .unwrap_or(
            "C:".to_string()
        );

    let system_drive: PathBuf = PathBuf::from(
        var_os("SYSTEMDRIVE")
            .unwrap_or(
                OsString::from("C:")
            )
    );

    let all_user_profile: PathBuf = PathBuf::from(
        var_os("ALLUSERSPROFILE")
            .unwrap_or(
                OsString::from(format!("{}\\ProgramData", system_drive_string))
            )
    );

    let system_root: PathBuf = PathBuf::from(
        var_os("SYSTEMROOT")
            .unwrap_or(
                OsString::from(
                    format!("{}\\Windows", &system_drive_string)
                )
            )
    );

    let sys32: PathBuf = system_root.join(PathBuf::from("System32"));

    let username: String = var("USERNAME")
        .unwrap_or(
            "Default".to_string()
        );

    let user_profile_string: String = var("USERPROFILE")
        .unwrap_or(
            format!("{}\\Users\\{}", system_drive_string, username)
        );

    let user_profile: PathBuf = PathBuf::from(
        var_os("USERPROFILE")
            .unwrap_or(
                OsString::from(&user_profile_string)
            )
    );

    let mut spt_user_profile: Vec<&str> = user_profile_string.split("\\").collect();
    spt_user_profile.truncate(spt_user_profile.len() - 1);

    let base_user_profile: PathBuf = PathBuf::from(spt_user_profile.join("\\"));

    check_path(&system_root, false, true);
    check_path(&sys32, false, true);
    check_path(&base_user_profile, false, true);
    check_path(&all_user_profile, false, true);
    check_path(&user_profile, false, true);

    system_vars.insert("system_drive", system_drive);
    system_vars.insert("system_root", system_root);
    system_vars.insert("sys32", sys32);
    system_vars.insert("base_user_profile", base_user_profile);
    system_vars.insert("all_user_profile", all_user_profile);
    system_vars.insert("user_profile", user_profile);

    system_vars
}

pub fn clean(old: bool) -> Result<(), Error> {
    adjust_privilege("SeRestorePrivilege");

    let system_vars: HashMap<&'static str, PathBuf> = get_system_vars();
    let users_profile_dirs: Vec<PathBuf> = get_users_dirs(&system_vars);
    let mut temp_components: Vec<TempComponent> = Vec::new();
    let local: DateTime<Local> = Local::now();
    let user_profile: PathBuf = PathBuf::from(var("USERPROFILE").unwrap_or("C:\\".to_string()));
    let local_datetime = local.format("%a %b %e %T %Y");
    let report: PathBuf = user_profile.join(format!(
        "Desktop\\KpTemp_{}.txt", local.format("%Y-%m-%d_%H-%M-%S").to_string()
    ));

    let nbr_row = 41 + 3;

    get_walk_all(&mut temp_components, &system_vars, &users_profile_dirs);
    get_walk_glob_file(&mut temp_components, &system_vars);
    get_glob_folders_clear(&mut temp_components, &users_profile_dirs);

    if old == true {
        windows_old(&mut temp_components, &system_vars);
    }

    let missing = (nbr_row - temp_components.len() as u64) * 2;

    for _ in 0..missing {
        thread::sleep(Duration::from_millis(50));
        advance_progress_bar(unsafe { PROGRESS_HANDLE }, 1);
    }

    let mut total_len = 0;
    let mut total_size = 0;

    for temp_component in &temp_components {
        total_size += temp_component.size;
        total_len += temp_component.len;
    }

    let mut output = File::create(report)?;

    output.write_all(format!("KpTemp v{} by kernel-panik\r\n", KPTEMP_VERSION).as_bytes())?;
    output.write_all(format!("Date: {}\r\n\r\n", local_datetime.to_string()).as_bytes())?;

    if 0 == temp_components.len() {
        output.write_all("No records found\n".as_bytes())?;
    }

    for mut temp_component in temp_components {
        walk(&mut temp_component, clear);
        walk(&mut temp_component, remove_on_reboot);

        output.write_all(format!(
            "{} : {} files => {} deleted\r\n",
            temp_component.path.display(),
            temp_component.len,
            convert(temp_component.size as f64)
        ).as_bytes())?;
    }

    let (r_len, r_size) = unsafe { data_recycle_bin() };

    total_len += r_len as u64;
    total_size += r_size as u64;

    output.write_all(format!(
        "Recycle Bin : {} files => {} deleted\r\n\r\n",
        r_len,
        convert(r_size as f64)
    ).as_bytes())?;

    output.write_all(format!(
        "Total : {} files => {} deleted\r\n",
        total_len,
        convert(total_size as f64)
    ).as_bytes())?;

    advance_progress_bar(unsafe { PROGRESS_HANDLE }, 1);
    unsafe { set_window_text(LABEL_HANDLE, "Clear recycle bin"); }
    empty_recycle_bin();
    advance_progress_bar(unsafe { PROGRESS_HANDLE }, TOTAL_STEP);
    message_box(format!("Success {} files and {} deleted", total_len, convert(total_size as f64)));

    Ok(())
}
