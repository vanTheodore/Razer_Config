//! Offline USB trace comparison — no USB operations or device drivers.
use cloncontrol_a704f::trace::{hex, only_in_action, parse_tshark, ParseResult};
use std::{env, fs};

fn show(label: &str, r: &ParseResult) {
    println!(
        "{label}: {} reportes HID, {} transferencias sin datos, {} descartadas, {} líneas problemáticas",
        r.transfers.len(), r.no_payload, r.discarded, r.malformed.len()
    );
    for item in &r.malformed {
        eprintln!("  {item}");
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 2 || matches!(args.first().map(String::as_str), Some("--help" | "-h")) {
        println!("CLON//CONTROL — comparador de trazas USB (solo lectura)");
        println!("Uso: a704f-trace.exe SIN_CAMBIOS.tsv CON_CAMBIO.tsv");
        println!("Exporta ambos archivos con TShark como se describe en docs/INSTANT_REFERENCE.md");
        return Ok(());
    }
    let baseline = fs::read_to_string(&args[0])
        .map_err(|e| format!("No se pudo leer {}: {e}", args[0]))?;
    let modified = fs::read_to_string(&args[1])
        .map_err(|e| format!("No se pudo leer {}: {e}", args[1]))?;
    let before = parse_tshark(&baseline);
    let after = parse_tshark(&modified);
    show("Referencia sin cambios", &before);
    show("Acción de la aplicación oficial", &after);
    println!("\nComandos presentes más veces en la segunda captura (NO verificados):");
    let candidates = only_in_action(&before.transfers, &after.transfers);
    if candidates.is_empty() {
        println!("Ninguna diferencia verificable; revisar el filtro o la captura.");
    } else {
        for (p, times) in &candidates {
            println!(
                "  Interfaz {}, {} Report ID {:02X}, +{} vez/veces, {} bytes: {}",
                p.interface,
                if p.report_type == 3 { "Feature" } else { "Output" },
                p.report_id, times, p.payload.len(), hex(&p.payload)
            );
        }
    }
    println!("\nAdvertencia: las diferencias no prueban que el software controle este mouse.");
    println!("El analizador nunca abre dispositivos HID ni envía comandos.");
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}
