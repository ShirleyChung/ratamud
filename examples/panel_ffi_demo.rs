//! 端到端驗證：透過真正的 C-ABI FFI 路徑（與 iOS Swift 端呼叫的完全相同）
//! 註冊 panel callback → init → 請求每個面板 → 真正買/賣，印出引擎推回的 ASCII。
//!
//! 執行： cargo run --no-default-features --example panel_ffi_demo

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Mutex;
use once_cell::sync::Lazy;

use ratamud::ffi::*;

static PANELS: Lazy<Mutex<Vec<(String, String)>>> = Lazy::new(|| Mutex::new(Vec::new()));

extern "C" fn panel_cb(panel: *const c_char, content: *const c_char) {
    let p = unsafe { CStr::from_ptr(panel) }.to_string_lossy().into_owned();
    let c = unsafe { CStr::from_ptr(content) }.to_string_lossy().into_owned();
    PANELS.lock().unwrap().push((p, c));
}

extern "C" fn out_cb(msg_type: *const c_char, content: *const c_char) {
    let t = unsafe { CStr::from_ptr(msg_type) }.to_string_lossy().into_owned();
    let c = unsafe { CStr::from_ptr(content) }.to_string_lossy().into_owned();
    if t == "MAIN" || t == "STATUS" {
        println!("    [{t}] {c}");
    }
}

fn drain_panels(header: &str) {
    println!("\n================ {header} ================");
    for (p, c) in PANELS.lock().unwrap().drain(..) {
        println!("---------- PANEL: {p} ----------");
        println!("{c}");
    }
}

fn cmd(s: &str) {
    let c = CString::new(s).unwrap();
    ratamud_input_command(c.as_ptr());
}

fn main() {
    ratamud_register_output_callback(out_cb);
    ratamud_register_panel_callback(panel_cb);

    let rc = ratamud_init_game();
    println!(">> ratamud_init_game() = {rc}");
    assert_eq!(rc, 0, "init failed");
    PANELS.lock().unwrap().clear();

    // ---- 請求每個唯讀面板 ----
    println!("\n>> 請求 MINIMAP / MAP / INVENTORY / STATUS / TRADE");
    assert_eq!(ratamud_request_minimap(), 0);
    assert_eq!(ratamud_request_map(), 0);
    assert_eq!(ratamud_request_inventory(), 0);
    assert_eq!(ratamud_request_status(), 0);
    let merchant = CString::new("商人").unwrap();
    assert_eq!(ratamud_request_trade(merchant.as_ptr()), 0);
    drain_panels("面板初始內容");

    // ---- 互動交易：買 / 賣 ----
    println!("\n>> 確保金幣充足：set me gold 99999");
    cmd("set me gold 99999");

    let npc = CString::new("商人").unwrap();
    let candidates = ["治療藥水", "木劍", "麵包", "舊布料", "匕首", "魔力藥水"];
    let mut bought: Option<&str> = None;
    for item in candidates {
        let item_c = CString::new(item).unwrap();
        let rc = ratamud_trade_buy(npc.as_ptr(), item_c.as_ptr(), 1);
        println!(">> ratamud_trade_buy(商人, {item}, 1) = {rc}  (1=成功 0=失敗 -1=錯誤)");
        if rc == 1 {
            bought = Some(item);
            break;
        }
        PANELS.lock().unwrap().clear();
    }
    drain_panels("買入成功後刷新的面板");

    if let Some(item) = bought {
        let item_c = CString::new(item).unwrap();
        let rc = ratamud_trade_sell(npc.as_ptr(), item_c.as_ptr(), 1);
        println!(">> ratamud_trade_sell(商人, {item}, 1) = {rc}");
        drain_panels("賣出後刷新的面板");
    } else {
        println!("!! 沒有任何候選物品買入成功（商人庫存可能不同）");
    }

    // ---- 錯誤情境：賣一個沒有的東西，應回 0 並推 STATUS 訊息 ----
    let bogus = CString::new("不存在的東西").unwrap();
    let rc = ratamud_trade_sell(npc.as_ptr(), bogus.as_ptr(), 1);
    println!("\n>> ratamud_trade_sell(商人, 不存在的東西, 1) = {rc}  (預期 0=失敗)");

    println!("\n✅ FFI 面板/交易路徑驗證完成");
}
