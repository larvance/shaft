use std::path::Path;
use calamine::{Data, Reader};
use shaft_core::normalize_format;

pub fn read_spreadsheet<P: AsRef<Path>>(path: P) -> Result<Vec<Vec<String>>, String> {
    let p = path.as_ref();
    if !p.exists() {
        return Err(format!("Spreadsheet file does not exist: {}", p.display()));
    }

    let ext = p
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "csv" => read_delimited(p, b','),
        "tsv" => read_delimited(p, b'\t'),
        "xlsx" | "xls" | "ods" | "xlsb" => read_workbook(p),
        _ => {
            // Attempt workbook reader first, fallback to CSV
            read_workbook(p).or_else(|_| read_delimited(p, b','))
        }
    }
}

fn read_delimited(path: &Path, delimiter: u8) -> Result<Vec<Vec<String>>, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(false)
        .flexible(true)
        .from_path(path)
        .map_err(|e| format!("Failed to open delimited file: {}", e))?;

    let mut grid = Vec::new();
    for result in rdr.records() {
        let record = result.map_err(|e| format!("Error reading record: {}", e))?;
        let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
        grid.push(row);
    }
    Ok(grid)
}

fn read_workbook(path: &Path) -> Result<Vec<Vec<String>>, String> {
    let mut workbook = calamine::open_workbook_auto(path)
        .map_err(|e| format!("Failed to open workbook: {}", e))?;

    let sheet_names = workbook.sheet_names().to_vec();
    if sheet_names.is_empty() {
        return Ok(Vec::new());
    }

    let range = workbook
        .worksheet_range(&sheet_names[0])
        .map_err(|e| format!("Failed to read sheet {}: {}", sheet_names[0], e))?;

    let mut grid = Vec::new();
    for row in range.rows() {
        let cells: Vec<String> = row
            .iter()
            .map(|c| match c {
                Data::Empty => String::new(),
                Data::String(s) => s.clone(),
                Data::Float(f) => {
                    if f.fract() == 0.0 && *f >= i64::MIN as f64 && *f <= i64::MAX as f64 {
                        format!("{}", *f as i64)
                    } else {
                        format!("{}", f)
                    }
                }
                Data::Int(i) => format!("{}", i),
                Data::Bool(b) => format!("{}", b),
                Data::DateTime(d) => format!("{}", d.as_f64()),
                Data::DateTimeIso(s) => s.clone(),
                Data::DurationIso(s) => s.clone(),
                Data::Error(e) => format!("{:?}", e),
            })
            .collect();
        grid.push(cells);
    }

    Ok(grid)
}

pub fn write_spreadsheet<P: AsRef<Path>>(
    grid: &[Vec<String>],
    path: P,
    target_format: &str,
) -> Result<(), String> {
    let dst = path.as_ref();
    if let Some(parent) = dst.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }

    let norm = normalize_format(target_format);

    match norm.as_str() {
        "csv" => write_delimited(grid, dst, b','),
        "tsv" => write_delimited(grid, dst, b'\t'),
        "xlsx" => write_xlsx(grid, dst),
        "txt" => write_delimited(grid, dst, b'\t'),
        "md" => write_markdown_table(grid, dst),
        _ => {
            // Default to CSV
            write_delimited(grid, dst, b',')
        }
    }
}

fn write_delimited(grid: &[Vec<String>], path: &Path, delimiter: u8) -> Result<(), String> {
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .from_path(path)
        .map_err(|e| format!("Failed to create delimited writer: {}", e))?;

    for row in grid {
        wtr.write_record(row)
            .map_err(|e| format!("Failed to write record: {}", e))?;
    }
    wtr.flush()
        .map_err(|e| format!("Failed to flush delimited output: {}", e))?;
    Ok(())
}

fn write_xlsx(grid: &[Vec<String>], path: &Path) -> Result<(), String> {
    let mut workbook = rust_xlsxwriter::Workbook::new();
    let worksheet = workbook.add_worksheet();

    for (row_idx, row) in grid.iter().enumerate() {
        for (col_idx, val) in row.iter().enumerate() {
            worksheet
                .write_string(row_idx as u32, col_idx as u16, val)
                .map_err(|e| format!("Failed to write cell: {}", e))?;
        }
    }

    workbook
        .save(path)
        .map_err(|e| format!("Failed to save XLSX: {}", e))?;
    Ok(())
}

fn write_markdown_table(grid: &[Vec<String>], path: &Path) -> Result<(), String> {
    if grid.is_empty() {
        std::fs::write(path, "").map_err(|e| e.to_string())?;
        return Ok(());
    }

    let col_count = grid.iter().map(|r| r.len()).max().unwrap_or(0);
    if col_count == 0 {
        std::fs::write(path, "").map_err(|e| e.to_string())?;
        return Ok(());
    }

    let mut out = String::new();

    // Header row
    let header = &grid[0];
    out.push('|');
    for i in 0..col_count {
        let val = header.get(i).map(|s| s.as_str()).unwrap_or("");
        out.push_str(&format!(" {} |", val));
    }
    out.push('\n');

    // Separator row
    out.push('|');
    for _ in 0..col_count {
        out.push_str(" --- |");
    }
    out.push('\n');

    // Data rows
    for row in grid.iter().skip(1) {
        out.push('|');
        for i in 0..col_count {
            let val = row.get(i).map(|s| s.as_str()).unwrap_or("");
            out.push_str(&format!(" {} |", val));
        }
        out.push('\n');
    }

    std::fs::write(path, out).map_err(|e| e.to_string())
}

pub fn transcode_spreadsheet<P: AsRef<Path>, Q: AsRef<Path>>(
    source_path: P,
    output_path: Q,
    target_format: &str,
) -> Result<(), String> {
    let grid = read_spreadsheet(source_path)?;
    write_spreadsheet(&grid, output_path, target_format)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_to_tsv_and_back() {
        let temp_dir = std::env::temp_dir();
        let src_csv = temp_dir.join("shaft_test_data.csv");
        let dst_tsv = temp_dir.join("shaft_test_data.tsv");
        let dst_csv = temp_dir.join("shaft_test_data_back.csv");

        let sample_csv = "Name,Age,Department\nAlice,30,Aerodynamics\nBob,25,Flight Control\n";
        std::fs::write(&src_csv, sample_csv).unwrap();

        // 1. CSV -> TSV
        let res1 = transcode_spreadsheet(&src_csv, &dst_tsv, "tsv");
        assert!(res1.is_ok(), "CSV to TSV failed: {:?}", res1);
        assert!(dst_tsv.exists());

        let tsv_content = std::fs::read_to_string(&dst_tsv).unwrap();
        assert!(tsv_content.contains("Alice\t30\tAerodynamics"));

        // 2. TSV -> CSV
        let res2 = transcode_spreadsheet(&dst_tsv, &dst_csv, "csv");
        assert!(res2.is_ok(), "TSV to CSV failed: {:?}", res2);
        assert!(dst_csv.exists());

        let grid = read_spreadsheet(&dst_csv).unwrap();
        assert_eq!(grid.len(), 3);
        assert_eq!(grid[1][0], "Alice");
        assert_eq!(grid[2][2], "Flight Control");

        let _ = std::fs::remove_file(&src_csv);
        let _ = std::fs::remove_file(&dst_tsv);
        let _ = std::fs::remove_file(&dst_csv);
    }

    #[test]
    fn test_csv_to_xlsx_and_markdown() {
        let temp_dir = std::env::temp_dir();
        let src_csv = temp_dir.join("shaft_table.csv");
        let dst_xlsx = temp_dir.join("shaft_table.xlsx");
        let dst_md = temp_dir.join("shaft_table.md");

        let sample_csv = "ID,Param,Value\n1,Thrust,1200\n2,Payload,450\n";
        std::fs::write(&src_csv, sample_csv).unwrap();

        // CSV -> XLSX
        let res_xlsx = transcode_spreadsheet(&src_csv, &dst_xlsx, "xlsx");
        assert!(res_xlsx.is_ok(), "CSV to XLSX failed: {:?}", res_xlsx);
        assert!(dst_xlsx.exists());
        assert!(std::fs::metadata(&dst_xlsx).unwrap().len() > 0);

        // Read XLSX back with calamine!
        let read_grid = read_spreadsheet(&dst_xlsx).unwrap();
        assert_eq!(read_grid.len(), 3);
        assert_eq!(read_grid[0][1], "Param");
        assert_eq!(read_grid[1][2], "1200");

        // CSV -> Markdown table
        let res_md = transcode_spreadsheet(&src_csv, &dst_md, "md");
        assert!(res_md.is_ok(), "CSV to Markdown table failed: {:?}", res_md);
        let md_content = std::fs::read_to_string(&dst_md).unwrap();
        assert!(md_content.contains("| ID | Param | Value |"));
        assert!(md_content.contains("| 1 | Thrust | 1200 |"));

        let _ = std::fs::remove_file(&src_csv);
        let _ = std::fs::remove_file(&dst_xlsx);
        let _ = std::fs::remove_file(&dst_md);
    }
}
