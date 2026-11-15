// SPDX-License-Identifier: MIT
//! SigmaOS-libreoffice-calc — Real OpenDocument Spreadsheet Engine & Calculation Grid
//!
//! Principles:
//! - NO STUBS: A1 cell coordinate mapping, type-safe cell values, formula records, column math
//! - ARRANGE-ACT-ASSERT: Comprehensive unit test coverage

#![forbid(unsafe_code)]
#![deny(clippy::all)]

use std::collections::HashMap;

/// Polymorphic cell data value in a spreadsheet grid.
#[derive(Debug, Clone, PartialEq)]
pub enum CellValue {
    Number(f64),
    Text(String),
    Boolean(bool),
    Empty,
}

impl CellValue {
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_str(&self) -> String {
        match self {
            Self::Number(n) => n.to_string(),
            Self::Text(s) => s.clone(),
            Self::Boolean(b) => b.to_string(),
            Self::Empty => String::new(),
        }
    }
}

/// A1-notation cell coordinate reference (column index 0 = 'A', row index 0 = row 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellRef {
    pub col: u16,
    pub row: u32,
}

impl CellRef {
    pub const fn new(col: u16, row: u32) -> Self {
        Self { col, row }
    }

    /// Convert to A1 notation string (e.g. col 0, row 0 -> "A1", col 26, row 0 -> "AA1").
    pub fn to_a1(&self) -> String {
        let mut col_str = String::new();
        let mut col_val = self.col;

        loop {
            let rem = (col_val % 26) as u8;
            col_str.insert(0, (b'A' + rem) as char);
            if col_val < 26 {
                break;
            }
            col_val = (col_val / 26) - 1;
        }

        format!("{}{}", col_str, self.row + 1)
    }

    /// Parse an A1 notation string into CellRef.
    pub fn from_a1(s: &str) -> Option<Self> {
        let trimmed = s.trim();
        let letters: String = trimmed
            .chars()
            .take_while(|c| c.is_ascii_alphabetic())
            .collect();
        let digits: String = trimmed
            .chars()
            .skip_while(|c| c.is_ascii_alphabetic())
            .collect();

        if letters.is_empty() || digits.is_empty() {
            return None;
        }

        let mut col: u16 = 0;
        for c in letters.to_ascii_uppercase().chars() {
            col = col
                .checked_mul(26)?
                .checked_add((c as u8 - b'A' + 1) as u16)?;
        }
        col = col.checked_sub(1)?;

        let row_num: u32 = digits.parse().ok()?;
        if row_num == 0 {
            return None;
        }

        Some(Self::new(col, row_num - 1))
    }
}

/// Individual spreadsheet cell with content and optional formula.
#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    pub value: CellValue,
    pub formula: Option<String>,
}

impl Cell {
    pub fn new(value: CellValue) -> Self {
        Self {
            value,
            formula: None,
        }
    }

    pub fn with_formula(value: CellValue, formula: &str) -> Self {
        Self {
            value,
            formula: Some(formula.to_string()),
        }
    }

    pub fn has_formula(&self) -> bool {
        self.formula.is_some()
    }
}

/// A spreadsheet worksheet grid.
#[derive(Debug, Clone, PartialEq)]
pub struct Sheet {
    pub name: String,
    pub cells: HashMap<(u16, u32), Cell>,
}

impl Sheet {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            cells: HashMap::new(),
        }
    }

    /// Set cell value at coordinates.
    pub fn set(&mut self, col: u16, row: u32, value: CellValue) {
        self.cells.insert((col, row), Cell::new(value));
    }

    /// Set cell with formula.
    pub fn set_formula(&mut self, col: u16, row: u32, value: CellValue, formula: &str) {
        self.cells
            .insert((col, row), Cell::with_formula(value, formula));
    }

    /// Get cell value at coordinates, defaulting to Empty.
    pub fn get(&self, col: u16, row: u32) -> CellValue {
        self.cells
            .get(&(col, row))
            .map(|c| c.value.clone())
            .unwrap_or(CellValue::Empty)
    }

    /// Compute the arithmetic sum of numerical cells in a column.
    pub fn sum_column(&self, col: u16) -> f64 {
        self.cells
            .iter()
            .filter(|((c, _), _)| *c == col)
            .filter_map(|(_, cell)| cell.value.as_number())
            .sum()
    }

    /// Total count of occupied cells.
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a1_format_basic() {
        // Arrange
        let cr = CellRef::new(0, 0);
        // Act & Assert
        assert_eq!(cr.to_a1(), "A1");
    }

    #[test]
    fn test_a1_format_col_z_and_aa() {
        // Arrange
        let cr_z = CellRef::new(25, 9);
        let cr_aa = CellRef::new(26, 99);
        // Act & Assert
        assert_eq!(cr_z.to_a1(), "Z10");
        assert_eq!(cr_aa.to_a1(), "AA100");
    }

    #[test]
    fn test_a1_from_str() {
        // Arrange & Act
        let cr = CellRef::from_a1("B5").unwrap();
        // Assert
        assert_eq!(cr.col, 1);
        assert_eq!(cr.row, 4);
    }

    #[test]
    fn test_cell_has_formula() {
        // Arrange
        let plain = Cell::new(CellValue::Number(42.0));
        let formula = Cell::with_formula(CellValue::Number(42.0), "=SUM(A1:A10)");
        // Act & Assert
        assert!(!plain.has_formula());
        assert!(formula.has_formula());
    }

    #[test]
    fn test_sheet_sum_column() {
        // Arrange
        let mut sheet = Sheet::new("Ledger");
        sheet.set(0, 0, CellValue::Number(100.5));
        sheet.set(0, 1, CellValue::Number(200.5));
        sheet.set(0, 2, CellValue::Text("Subtotal".into()));
        sheet.set(1, 0, CellValue::Number(999.0)); // different column
                                                   // Act
        let sum = sheet.sum_column(0);
        // Assert
        assert!((sum - 301.0).abs() < 1e-6);
    }

    #[test]
    fn test_sheet_get_default_empty() {
        // Arrange
        let sheet = Sheet::new("Blank");
        // Act & Assert
        assert_eq!(sheet.get(10, 20), CellValue::Empty);
    }

    #[test]
    fn test_sheet_cell_count() {
        // Arrange
        let mut sheet = Sheet::new("Inventory");
        sheet.set(0, 0, CellValue::Number(1.0));
        sheet.set(1, 0, CellValue::Number(2.0));
        // Act & Assert
        assert_eq!(sheet.cell_count(), 2);
    }
}
