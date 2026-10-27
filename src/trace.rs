//! Read-only analysis of TShark USB control-transfer field exports.
//! This module does not open HID devices, load capture drivers, or send USB data.
//! We deliberately retain raw payload bytes and never silently treat a setup
//! prefix as part of an HID report. Wireshark layouts differ across versions.

use std::collections::HashMap;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Fingerprint {
    /// HID Output (2) or Feature (3) report type from wValue's high byte.
    pub report_type: u8,
    pub report_id: u8,
    pub interface: u16,
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transfer {
    pub frame: String,
    pub fingerprint: Fingerprint,
}

#[derive(Debug, Default)]
pub struct ParseResult {
    pub transfers: Vec<Transfer>,
    pub discarded: usize,
    pub no_payload: usize,
    pub malformed: Vec<String>,
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|x| format!("{x:02X}")).collect::<Vec<_>>().join(" ")
}

/// Numeric Wireshark fields are usually formatted with 0x prefixes.
/// Plain numbers are interpreted as decimal, not guessed to be hex.
fn number(field: &str) -> Option<u16> {
    let v = field.trim();
    if let Some(rest) = v.strip_prefix("0x").or_else(|| v.strip_prefix("0X")) {
        u16::from_str_radix(rest, 16).ok()
    } else {
        v.parse::<u16>().ok()
    }
}

/// Accept "07:13:7f", "07 13 7f", or "07137F" as hex byte strings.
fn bytes(field: &str) -> Option<Vec<u8>> {
    let mut compact = String::new();
    for c in field.trim().chars() {
        if c.is_ascii_hexdigit() {
            compact.push(c);
        } else if !matches!(c, ':' | ' ' | ',' | '-') {
            return None;
        }
    }
    if compact.is_empty() || compact.len() % 2 != 0 { return None; }
    compact.as_bytes().chunks(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok())
        .collect()
}

/// Parse 6 tab-separated TShark fields, in this exact order:
/// frame.number, usb.setup.bRequest, usb.setup.wValue, usb.setup.wIndex,
/// usb.capdata, usb.data_fragment
///
/// Keep only HID Set_Report transfers (bRequest 9, wValue type 2 or 3).
/// No device address, paths or serials are required or stored.
pub fn parse_tshark(source: &str) -> ParseResult {
    let mut result = ParseResult::default();
    for (lineno, line) in source.lines().enumerate() {
        if line.trim().is_empty() { continue; }
        let parts: Vec<&str> = line.trim_end_matches('\r').split('\t').collect();
        if parts.len() != 6 {
            result.malformed.push(format!("Línea {}: se necesitan 6 columnas TSV (hay {})", lineno + 1, parts.len()));
            continue;
        }
        if parts[0].trim() == "frame.number" { continue; }
        let (Some(request), Some(w_value), Some(index)) =
            (number(parts[1]), number(parts[2]), number(parts[3]))
        else {
            if parts[1].trim().is_empty() { result.discarded += 1; }
            else { result.malformed.push(format!("Línea {}: campos USB numéricos inválidos", lineno + 1)); }
            continue;
        };
        let report_type = (w_value >> 8) as u8;
        if request != 9 || !matches!(report_type, 2 | 3) {
            result.discarded += 1;
            continue;
        }
        // Prefer the explicitly dissected data fragment over leftover capture data.
        // Never concatenate both: some Wireshark versions expose the same bytes twice.
        let raw = if !parts[5].trim().is_empty() { parts[5] } else { parts[4] };
        if raw.trim().is_empty() { result.no_payload += 1; continue; }
        let Some(payload) = bytes(raw) else {
            result.malformed.push(format!("Línea {}: datos hexadecimales ilegibles", lineno + 1));
            continue;
        };
        result.transfers.push(Transfer {
            frame: parts[0].trim().to_string(),
            fingerprint: Fingerprint {
                report_type,
                report_id: w_value as u8,
                interface: index,
                payload,
            },
        });
    }
    result
}

pub fn count(transfers: &[Transfer]) -> HashMap<Fingerprint, usize> {
    let mut result = HashMap::new();
    for transfer in transfers {
        *result.entry(transfer.fingerprint.clone()).or_insert(0) += 1;
    }
    result
}

/// Multiset delta: a newly observed command has an increased occurrence count.
pub fn only_in_action(idle: &[Transfer], action: &[Transfer]) -> Vec<(Fingerprint, usize)> {
    let reference = count(idle);
    let mut diff: Vec<_> = count(action)
        .into_iter()
        .filter_map(|(packet, total)| {
            let previous = reference.get(&packet).copied().unwrap_or(0);
            (total > previous).then_some((packet, total - previous))
        })
        .collect();
    diff.sort_by(|a, b| (
        a.0.interface, a.0.report_type, a.0.report_id, &a.0.payload
    ).cmp(&(b.0.interface, b.0.report_type, b.0.report_id, &b.0.payload)));
    diff
}

#[cfg(test)]
mod tests {
    use super::*;
    const IDLE: &str = "12\t0x09\t0x0307\t0x0001\t07:13:7f:10:00:00:00:00\t\n";
    const ACTION: &str = concat!(
        "22\t0x09\t0x0307\t0x0001\t07:13:7f:10:00:00:00:00\t\n",
        "24\t0x09\t0x0307\t0x0001\t07:13:7f:17:00:00:00:00\t\n"
    );
    #[test]
    fn decode_only_hid_set_report_and_diff() {
        let idle = parse_tshark(IDLE);
        let action = parse_tshark(ACTION);
        assert!(idle.malformed.is_empty());
        assert!(action.malformed.is_empty());
        assert_eq!(idle.transfers.len(), 1);
        let diff = only_in_action(&idle.transfers, &action.transfers);
        assert_eq!(diff.len(), 1);
        assert_eq!(diff[0].0.report_type, 3);
        assert_eq!(diff[0].0.report_id, 7);
        assert_eq!(diff[0].0.interface, 1);
        assert_eq!(diff[0].0.payload, vec![7, 0x13, 0x7f, 0x17, 0, 0, 0, 0]);
    }
    #[test]
    fn prefer_fragment_and_support_hex_stream() {
        let input = "4\t9\t775\t1\tdeadbeef\t07137f1700000000\n";
        let r = parse_tshark(input);
        assert_eq!(r.transfers.len(), 1);
        assert_eq!(hex(&r.transfers[0].fingerprint.payload), "07 13 7F 17 00 00 00 00");
    }
    #[test]
    fn ignore_interrupt_and_missing_payload_without_faking_matches() {
        let text = concat!(
            "frame.number\tusb.setup.bRequest\tusb.setup.wValue\tusb.setup.wIndex\tusb.capdata\tusb.data_fragment\n",
            "1\t0x01\t0x0307\t0x0001\t07:13:7f:17:00:00:00:00\t\n",
            "2\t0x09\t0x0307\t0x0001\t\t\n",
            "3\t\t\t\t00:01\t\n"
        );
        let r = parse_tshark(text);
        assert!(r.transfers.is_empty());
        assert_eq!(r.discarded, 2);
        assert_eq!(r.no_payload, 1);
    }
    #[test]
    fn rejection_and_counting_are_deterministic() {
        let r = parse_tshark("3\t0x09\t0x0207\t0x0001\tzz\t\n");
        assert_eq!(r.malformed.len(), 1);
        let action = parse_tshark(&format!("{ACTION}{ACTION}"));
        let idle = parse_tshark(ACTION);
        let diff = only_in_action(&idle.transfers, &action.transfers);
        assert_eq!(diff.len(), 2);
        assert_eq!(diff[0].1, 1);
    }
}
