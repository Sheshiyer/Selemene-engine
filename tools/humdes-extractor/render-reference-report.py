#!/usr/bin/env python3
"""
render-reference-report.py
Converts a Selemene reference-report Markdown file into a polished DOCX.

Usage
-----
  python3 render-reference-report.py --markdown <path.md> --output <path.docx> \
      [--title "Custom Title"] [--language en|fr]

Only the bundled Python runtime is required:
  /Users/sheshnarayaniyer/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3

Design reference (observed US Letter, editorially documented)
-------------------------------------------------------------
  Page   : US Letter (8.5 × 11 in)
  Margins: top/bottom 1 in, left/right 0.9 in
  Body   : Calibri 11 pt, 1.08 line-spacing, 5 pt space-after
  H1     : Calibri 17 pt Bold, warm maroon #722F37, 18 pt before, 6 pt after, no outline
  H2     : Calibri 13 pt Bold, warm maroon #722F37, 12 pt before, 4 pt after
  H3     : Calibri 11.5 pt Bold, dark grey #333333, 8 pt before, 3 pt after
  Table  : repeating header row, Calibri 10 pt, header fill #722F37 text white,
           alternating row shading #F5F0EE / white, no row splits, keep-with-next
  TOC    : static Heading-style lines, page-numbers omitted (nav aid only)
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import sys
import textwrap
from copy import deepcopy
from pathlib import Path
from typing import Iterator, List, Optional, Tuple

# ---------------------------------------------------------------------------
# runtime guard — must be bundled Python with python-docx
# ---------------------------------------------------------------------------
try:
    from docx import Document
    from docx.enum.table import WD_TABLE_ALIGNMENT, WD_ALIGN_VERTICAL
    from docx.enum.text import WD_ALIGN_PARAGRAPH, WD_LINE_SPACING
    from docx.oxml import OxmlElement
    from docx.oxml.ns import qn
    from docx.shared import Inches, Pt, RGBColor, Emu
except ImportError as exc:
    sys.exit(
        f"[render-reference-report] python-docx not found: {exc}\n"
        "Use the bundled runtime:\n"
        "  /Users/sheshnarayaniyer/.cache/codex-runtimes/"
        "codex-primary-runtime/dependencies/python/bin/python3"
    )

# ---------------------------------------------------------------------------
# constants
# ---------------------------------------------------------------------------
MAROON      = RGBColor(0x72, 0x2F, 0x37)
DARK_GREY   = RGBColor(0x33, 0x33, 0x33)
WHITE       = RGBColor(0xFF, 0xFF, 0xFF)
ROW_ALT     = "F5F0EE"   # hex, no #
ROW_PLAIN   = "FFFFFF"
FONT_MAIN   = "Calibri"

PAGE_W      = Inches(8.5)
PAGE_H      = Inches(11.0)
MARGIN_TB   = Inches(1.0)
MARGIN_LR   = Inches(0.9)

# ---------------------------------------------------------------------------
# XML helpers
# ---------------------------------------------------------------------------

def _set_run_color(run, color: RGBColor) -> None:
    rpr = run._r.get_or_add_rPr()
    color_el = OxmlElement("w:color")
    color_el.set(qn("w:val"), f"{color.red:02X}{color.green:02X}{color.blue:02X}")
    rpr.append(color_el)


def _set_para_shading(para, fill_hex: str) -> None:
    ppr = para._p.get_or_add_pPr()
    shd = OxmlElement("w:shd")
    shd.set(qn("w:val"), "clear")
    shd.set(qn("w:color"), "auto")
    shd.set(qn("w:fill"), fill_hex)
    ppr.append(shd)


def _set_cell_shading(cell, fill_hex: str) -> None:
    tc = cell._tc
    tcPr = tc.get_or_add_tcPr()
    shd = OxmlElement("w:shd")
    shd.set(qn("w:val"), "clear")
    shd.set(qn("w:color"), "auto")
    shd.set(qn("w:fill"), fill_hex)
    tcPr.append(shd)


def _set_cell_no_split(cell) -> None:
    tc = cell._tc
    tcPr = tc.get_or_add_tcPr()
    noWrap = OxmlElement("w:noWrap")
    tcPr.append(noWrap)


def _enable_repeat_header(row) -> None:
    trPr = row._tr.get_or_add_trPr()
    tblHeader = OxmlElement("w:tblHeader")
    trPr.append(tblHeader)


def _keep_row_together(row) -> None:
    trPr = row._tr.get_or_add_trPr()
    cantSplit = OxmlElement("w:cantSplit")
    cantSplit.set(qn("w:val"), "1")
    trPr.append(cantSplit)


def _set_table_borders(table) -> None:
    tbl = table._tbl
    tblPr = tbl.find(qn("w:tblPr"))
    if tblPr is None:
        tblPr = OxmlElement("w:tblPr")
        tbl.insert(0, tblPr)
    tblBorders = OxmlElement("w:tblBorders")
    for side in ("top", "left", "bottom", "right", "insideH", "insideV"):
        el = OxmlElement(f"w:{side}")
        el.set(qn("w:val"), "single")
        el.set(qn("w:sz"), "4")
        el.set(qn("w:space"), "0")
        el.set(qn("w:color"), "BFBFBF")
        tblBorders.append(el)
    existing = tblPr.find(qn("w:tblBorders"))
    if existing is not None:
        tblPr.remove(existing)
    tblPr.append(tblBorders)


def _set_table_layout_fixed(table) -> None:
    tbl = table._tbl
    tblPr = tbl.find(qn("w:tblPr"))
    if tblPr is None:
        tblPr = OxmlElement("w:tblPr")
        tbl.insert(0, tblPr)
    tblLayout = OxmlElement("w:tblLayout")
    tblLayout.set(qn("w:type"), "fixed")
    tblPr.append(tblLayout)


def _set_col_widths(table, widths_twips: List[int]) -> None:
    """Set explicit column widths in twips (1 inch = 1440 twips)."""
    tbl = table._tbl
    tblGrid = tbl.find(qn("w:tblGrid"))
    if tblGrid is not None:
        tbl.remove(tblGrid)
    tblGrid = OxmlElement("w:tblGrid")
    for w in widths_twips:
        gc = OxmlElement("w:gridCol")
        gc.set(qn("w:w"), str(w))
        tblGrid.append(gc)
    tbl.insert(1, tblGrid)
    for row in table.rows:
        for idx, cell in enumerate(row.cells):
            if idx < len(widths_twips):
                tc = cell._tc
                tcPr = tc.get_or_add_tcPr()
                tcW = OxmlElement("w:tcW")
                tcW.set(qn("w:w"), str(widths_twips[idx]))
                tcW.set(qn("w:type"), "dxa")
                existing = tcPr.find(qn("w:tcW"))
                if existing is not None:
                    tcPr.remove(existing)
                tcPr.insert(0, tcW)


def _set_tbl_width(table, width_twips: int) -> None:
    tbl = table._tbl
    tblPr = tbl.find(qn("w:tblPr"))
    if tblPr is None:
        tblPr = OxmlElement("w:tblPr")
        tbl.insert(0, tblPr)
    tblW = OxmlElement("w:tblW")
    tblW.set(qn("w:w"), str(width_twips))
    tblW.set(qn("w:type"), "dxa")
    existing = tblPr.find(qn("w:tblW"))
    if existing is not None:
        tblPr.remove(existing)
    tblPr.append(tblW)


def _page_number_field(para) -> None:
    """Insert PAGE field into para."""
    run = para.add_run()
    fldChar1 = OxmlElement("w:fldChar")
    fldChar1.set(qn("w:fldCharType"), "begin")
    instrText = OxmlElement("w:instrText")
    instrText.set(qn("xml:space"), "preserve")
    instrText.text = " PAGE "
    fldChar2 = OxmlElement("w:fldChar")
    fldChar2.set(qn("w:fldCharType"), "end")
    run._r.append(fldChar1)
    run._r.append(instrText)
    run._r.append(fldChar2)


def _numpages_field(para) -> None:
    """Insert NUMPAGES field into para."""
    run = para.add_run()
    fldChar1 = OxmlElement("w:fldChar")
    fldChar1.set(qn("w:fldCharType"), "begin")
    instrText = OxmlElement("w:instrText")
    instrText.set(qn("xml:space"), "preserve")
    instrText.text = " NUMPAGES "
    fldChar2 = OxmlElement("w:fldChar")
    fldChar2.set(qn("w:fldCharType"), "end")
    run._r.append(fldChar1)
    run._r.append(instrText)
    run._r.append(fldChar2)


# ---------------------------------------------------------------------------
# Markdown parser
# ---------------------------------------------------------------------------

class _Block:
    """Parsed block of Markdown content."""
    __slots__ = ("kind", "level", "text", "rows", "alignments")

    def __init__(self, kind: str, level: int = 0, text: str = "",
                 rows: Optional[List[List[str]]] = None,
                 alignments: Optional[List[str]] = None):
        self.kind = kind          # heading|para|table|rule|blank|code
        self.level = level        # 1-3 for headings
        self.text = text          # raw text
        self.rows = rows or []    # list-of-lists for tables
        self.alignments = alignments or []


_HEADING_RE = re.compile(r"^(#{1,6})\s+(.+)$")
_SEPARATOR_RE = re.compile(r"^[\s\|:\-]+$")
_TABLE_ROW_RE = re.compile(r"^\|(.+)\|$")
_RULE_RE = re.compile(r"^---+\s*$")
_CODE_FENCE_RE = re.compile(r"^```")


def _parse_table_row(line: str) -> List[str]:
    cells = line.strip().strip("|").split("|")
    return [c.strip() for c in cells]


def _is_separator_row(cells: List[str]) -> bool:
    return all(re.match(r"^:?-+:?$", c.strip()) for c in cells if c.strip())


def _parse_markdown(text: str) -> List[_Block]:
    lines = text.splitlines()
    blocks: List[_Block] = []
    i = 0
    in_code = False

    while i < len(lines):
        line = lines[i]

        # code fence toggle
        if _CODE_FENCE_RE.match(line):
            in_code = not in_code
            if in_code:
                code_lines = []
                i += 1
                while i < len(lines) and not _CODE_FENCE_RE.match(lines[i]):
                    code_lines.append(lines[i])
                    i += 1
                in_code = False
                blocks.append(_Block("code", text="\n".join(code_lines)))
                i += 1
                continue
            i += 1
            continue

        if in_code:
            i += 1
            continue

        # heading
        m = _HEADING_RE.match(line)
        if m:
            level = min(len(m.group(1)), 3)
            blocks.append(_Block("heading", level=level, text=m.group(2).strip()))
            i += 1
            continue

        # horizontal rule
        if _RULE_RE.match(line):
            blocks.append(_Block("rule"))
            i += 1
            continue

        # blank line
        if not line.strip():
            blocks.append(_Block("blank"))
            i += 1
            continue

        # table (detect by leading pipe or next line being separator)
        if _TABLE_ROW_RE.match(line):
            table_lines = []
            while i < len(lines) and (
                _TABLE_ROW_RE.match(lines[i]) or
                (lines[i].strip().startswith("|") and lines[i].strip().endswith("|"))
            ):
                table_lines.append(lines[i])
                i += 1
            # parse table_lines into header + rows
            parsed_rows: List[List[str]] = []
            alignments: List[str] = []
            header_row: Optional[List[str]] = None
            for tline in table_lines:
                cells = _parse_table_row(tline)
                if _is_separator_row(cells):
                    # derive alignment
                    for c in cells:
                        c = c.strip()
                        if c.startswith(":") and c.endswith(":"):
                            alignments.append("center")
                        elif c.endswith(":"):
                            alignments.append("right")
                        else:
                            alignments.append("left")
                    continue
                if header_row is None:
                    header_row = cells
                    parsed_rows.append(cells)
                else:
                    parsed_rows.append(cells)
            blocks.append(_Block("table", rows=parsed_rows, alignments=alignments))
            continue

        # paragraph — collect until blank/heading/table
        para_lines = [line]
        i += 1
        while i < len(lines):
            nxt = lines[i]
            if (not nxt.strip() or
                    _HEADING_RE.match(nxt) or
                    _TABLE_ROW_RE.match(nxt) or
                    _RULE_RE.match(nxt) or
                    _CODE_FENCE_RE.match(nxt)):
                break
            para_lines.append(nxt)
            i += 1
        blocks.append(_Block("para", text=" ".join(para_lines)))

    return blocks


# ---------------------------------------------------------------------------
# Inline formatting (bold, italic, code, link)
# ---------------------------------------------------------------------------

_INLINE_RE = re.compile(
    r"(\*\*\*(.+?)\*\*\*"          # bold+italic
    r"|\*\*(.+?)\*\*"               # bold
    r"|(?<!\w)__(?!\s)(.+?)(?<!\s)__(?!\w)"                   # bold alt
    r"|\*(.+?)\*"                   # italic
    r"|(?<!\w)_(?!\s)(.+?)(?<!\s)_(?!\w)"                     # italic alt
    r"|`([^`]+)`"                   # inline code
    r"|\[([^\]]+)\]\([^)]*\)"       # link — keep label
    r")"
)


def _apply_inline(paragraph, text: str, base_size_pt: float = 11,
                  base_bold: bool = False, base_italic: bool = False) -> None:
    """Parse inline MD tokens and add styled runs to paragraph."""
    pos = 0
    for m in _INLINE_RE.finditer(text):
        # plain text before match
        if m.start() > pos:
            run = paragraph.add_run(text[pos:m.start()])
            run.font.name = FONT_MAIN
            run.font.size = Pt(base_size_pt)
            if base_bold:
                run.bold = True
            run.italic = base_italic

        full = m.group(0)
        if full.startswith(("*", "_")):
            inner = next(value for value in m.groups()[1:6] if value is not None)
            bold = full.startswith(("**", "__"))
            italic = full.startswith("***") or not bold
            _apply_inline(paragraph, inner, base_size_pt, base_bold or bold, base_italic or italic)
            pos = m.end()
            continue
        if full.startswith("***"):
            content = m.group(2)
            run = paragraph.add_run(content)
            run.bold = True
            run.italic = True
        elif full.startswith("**") or full.startswith("__"):
            content = m.group(3) or m.group(4)
            run = paragraph.add_run(content)
            run.bold = True
        elif full.startswith("*") or full.startswith("_"):
            content = m.group(5) or m.group(6)
            run = paragraph.add_run(content)
            run.italic = True
        elif full.startswith("`"):
            content = m.group(7)
            run = paragraph.add_run(content)
            run.font.name = "Courier New"
            run.font.size = Pt(9.5)
        else:
            # link — keep label text
            content = m.group(8)
            run = paragraph.add_run(content)

        run.font.name = FONT_MAIN if not full.startswith("`") else "Courier New"
        run.font.size = Pt(min(base_size_pt, 9.5) if full.startswith("`") else base_size_pt)
        run.italic = base_italic or bool(run.italic)
        if base_bold:
            run.bold = run.bold or True

        pos = m.end()

    # remaining text
    if pos < len(text):
        run = paragraph.add_run(text[pos:])
        run.font.name = FONT_MAIN
        run.font.size = Pt(base_size_pt)
        if base_bold:
            run.bold = True
        run.italic = base_italic


# ---------------------------------------------------------------------------
# Document builder
# ---------------------------------------------------------------------------

class ReportBuilder:
    """Builds a DOCX from parsed Markdown blocks."""

    # usable page width in twips (1 in = 1440 twips)
    BODY_WIDTH_IN   = 8.5 - 0.9 * 2   # 6.7 in
    BODY_WIDTH_DXA  = int(BODY_WIDTH_IN * 1440)  # 9648

    def __init__(self, title: str, language: str):
        self.title    = title
        self.language = language.lower()
        self.doc      = Document()
        self._setup_document()

    # ------------------------------------------------------------------
    # document-level setup
    # ------------------------------------------------------------------

    def _setup_document(self) -> None:
        doc = self.doc
        # page size + margins
        for section in doc.sections:
            section.page_width   = PAGE_W
            section.page_height  = PAGE_H
            section.top_margin   = MARGIN_TB
            section.bottom_margin = MARGIN_TB
            section.left_margin  = MARGIN_LR
            section.right_margin = MARGIN_LR
            self._add_page_numbers(section)

        # default paragraph style
        style = doc.styles["Normal"]
        style.font.name  = FONT_MAIN
        style.font.size  = Pt(11)
        pf = style.paragraph_format
        pf.space_after   = Pt(5)
        pf.line_spacing_rule = WD_LINE_SPACING.MULTIPLE
        pf.line_spacing  = 1.08

        # heading styles
        self._configure_heading("Heading 1", 17, MAROON, space_before=18, space_after=6)
        self._configure_heading("Heading 2", 13, MAROON, space_before=12, space_after=4)
        self._configure_heading("Heading 3", 11.5, DARK_GREY, space_before=8, space_after=3)

    def _configure_heading(self, style_name: str, size_pt: float,
                            color: RGBColor, space_before: int,
                            space_after: int) -> None:
        style = self.doc.styles[style_name]
        style.font.name  = FONT_MAIN
        style.font.size  = Pt(size_pt)
        style.font.bold  = True
        style.font.color.rgb = color
        pf = style.paragraph_format
        pf.space_before  = Pt(space_before)
        pf.space_after   = Pt(space_after)
        pf.keep_with_next = True

    def _add_page_numbers(self, section) -> None:
        """Add centered page numbers in the footer."""
        footer = section.footer
        para = footer.paragraphs[0] if footer.paragraphs else footer.add_paragraph()
        para.alignment = WD_ALIGN_PARAGRAPH.CENTER
        para.clear()
        para.add_run("— ")
        _page_number_field(para)
        para.add_run(" —")
        for run in para.runs:
            run.font.name = FONT_MAIN
            run.font.size = Pt(9)
            run.font.color.rgb = DARK_GREY

    # ------------------------------------------------------------------
    # cover page
    # ------------------------------------------------------------------

    def add_cover(self) -> None:
        doc = self.doc
        # spacer
        for _ in range(4):
            p = doc.add_paragraph()
            p.paragraph_format.space_after = Pt(0)

        # document title
        title_para = doc.add_paragraph()
        title_para.alignment = WD_ALIGN_PARAGRAPH.CENTER
        run = title_para.add_run(self.title)
        run.font.name = FONT_MAIN
        run.font.size = Pt(22)
        run.font.bold = True
        run.font.color.rgb = MAROON

        # subtitle / meta
        lang_label = "English" if self.language == "en" else "Français"
        sub = doc.add_paragraph()
        sub.alignment = WD_ALIGN_PARAGRAPH.CENTER
        sub_run = sub.add_run(f"Selemene Integrated Reading  ·  {lang_label}")
        sub_run.font.name = FONT_MAIN
        sub_run.font.size = Pt(12)
        sub_run.font.color.rgb = DARK_GREY
        sub_run.italic = True

        doc.add_page_break()

    # ------------------------------------------------------------------
    # TOC (static)
    # ------------------------------------------------------------------

    def add_toc(self, headings: List[Tuple[int, str]]) -> None:
        doc = self.doc
        toc_title = doc.add_paragraph()
        toc_title.alignment = WD_ALIGN_PARAGRAPH.LEFT
        run = toc_title.add_run("Contents" if self.language == "en" else "Sommaire")
        run.font.name  = FONT_MAIN
        run.font.size  = Pt(14)
        run.font.bold  = True
        run.font.color.rgb = MAROON
        toc_title.paragraph_format.space_before = Pt(0)
        toc_title.paragraph_format.space_after  = Pt(6)

        for level, text in headings:
            p = doc.add_paragraph()
            p.paragraph_format.space_before = Pt(0)
            p.paragraph_format.space_after  = Pt(2)
            p.paragraph_format.line_spacing_rule = WD_LINE_SPACING.MULTIPLE
            p.paragraph_format.line_spacing = 1.1
            indent = Inches(0.25 * (level - 1))
            p.paragraph_format.left_indent = indent
            run = p.add_run(text)
            run.font.name = FONT_MAIN
            run.font.size = Pt(10 if level == 1 else 9)
            run.font.bold = (level == 1)
            run.font.color.rgb = MAROON if level == 1 else DARK_GREY

        doc.add_page_break()

    # ------------------------------------------------------------------
    # content block rendering
    # ------------------------------------------------------------------

    def render_heading(self, level: int, text: str) -> None:
        style_map = {1: "Heading 1", 2: "Heading 2", 3: "Heading 3"}
        style = style_map.get(level, "Heading 3")
        p = self.doc.add_paragraph(style=style)
        p.clear()
        # Each major chapter starts on a fresh page in either report language.
        # Numeric subsection headings keep their ordinary continuous flow.
        if level <= 2 and re.match(r"^(?:Part|Partie)\s+[IVX]+\b", text):
            p.paragraph_format.page_break_before = True
        _apply_inline(p, text, base_size_pt=17 if level == 1 else (13 if level == 2 else 11.5),
                      base_bold=True)
        # recolor all runs
        color = MAROON if level <= 2 else DARK_GREY
        for run in p.runs:
            run.font.color.rgb = color

    def render_paragraph(self, text: str) -> None:
        closing = re.match(
            r"^(?:\*{1,2})?(?:One open (?:reflection )?question|Question ouverte|"
            r"An optional (?:inquiry|question)|Can the chart remain a mirror)", text
        )
        if closing and self.doc.paragraphs:
            # Keep a chapter's reflection with the preceding discussion instead
            # of producing a nearly empty page containing its last few lines.
            for preceding in reversed(self.doc.paragraphs):
                preceding.paragraph_format.keep_with_next = True
                if preceding.text.strip():
                    break
        p = self.doc.add_paragraph()
        p.paragraph_format.widow_control = True
        if closing:
            p.paragraph_format.keep_together = True
        _apply_inline(p, text, base_size_pt=11)

    def render_code(self, text: str) -> None:
        p = self.doc.add_paragraph()
        p.paragraph_format.left_indent  = Inches(0.3)
        p.paragraph_format.right_indent = Inches(0.3)
        p.paragraph_format.space_after  = Pt(6)
        _set_para_shading(p, "F0EEEC")
        run = p.add_run(text)
        run.font.name = "Courier New"
        run.font.size = Pt(9)

    def render_rule(self) -> None:
        p = self.doc.add_paragraph()
        pPr = p._p.get_or_add_pPr()
        pBdr = OxmlElement("w:pBdr")
        bottom = OxmlElement("w:bottom")
        bottom.set(qn("w:val"), "single")
        bottom.set(qn("w:sz"), "4")
        bottom.set(qn("w:space"), "1")
        bottom.set(qn("w:color"), "BFBFBF")
        pBdr.append(bottom)
        pPr.append(pBdr)
        p.paragraph_format.space_after = Pt(8)

    def render_table(self, rows: List[List[str]],
                     alignments: List[str]) -> None:
        if not rows:
            return

        ncols = max(len(r) for r in rows)
        # normalise rows to same column count
        norm_rows = [r + [""] * (ncols - len(r)) for r in rows]

        # Column-width heuristic: distribute BODY_WIDTH_DXA proportionally
        # based on max content length per column
        col_max_lens = [0] * ncols
        for row in norm_rows:
            for ci, cell in enumerate(row):
                col_max_lens[ci] = max(col_max_lens[ci], len(cell))
        # Reserve a readable minimum before distributing extra width. Linear
        # character weighting made short system names collapse into 3-letter lines.
        compact = [ncols > 6 and all(len(row[i].strip('*` ')) <= 3 for row in norm_rows[1:]) for i in range(ncols)]
        floors = [max(650, len(norm_rows[0][i]) * 95 + 260) if compact[i] else (1440 if ncols <= 3 else 900 if ncols > 6 else 1080) for i in range(ncols)]
        if sum(floors) > self.BODY_WIDTH_DXA:
            floors = [int(self.BODY_WIDTH_DXA / ncols)] * ncols
        # Narrative comparison columns need room for paragraphs; the ordinary
        # square-root cap treats a 900-character synthesis like a short label.
        narrative = ncols >= 4 and max(col_max_lens) > 400
        weights = [0 if compact[i] else max(1, length if narrative else min(length, 160)) ** (0.85 if narrative else 0.5) for i, length in enumerate(col_max_lens)]
        if not sum(weights):
            weights = [1] * ncols
        available = self.BODY_WIDTH_DXA - sum(floors)
        col_widths = [int(floor + available * weight / sum(weights)) for floor, weight in zip(floors, weights)]
        col_widths[-1] += self.BODY_WIDTH_DXA - sum(col_widths)

        table = self.doc.add_table(rows=0, cols=ncols)
        table.autofit = False
        table.alignment = WD_TABLE_ALIGNMENT.LEFT
        _set_table_borders(table)
        _set_table_layout_fixed(table)
        _set_tbl_width(table, self.BODY_WIDTH_DXA)

        for ri, row_data in enumerate(norm_rows):
            row_obj = table.add_row()
            _keep_row_together(row_obj)
            if ri == 0:
                _enable_repeat_header(row_obj)

            for ci, cell_text in enumerate(row_data):
                cell = row_obj.cells[ci]
                if ri == 0:
                    _set_cell_shading(cell, "722F37")
                elif ri % 2 == 0:
                    _set_cell_shading(cell, ROW_ALT)
                else:
                    _set_cell_shading(cell, ROW_PLAIN)

                # alignment
                align = WD_ALIGN_PARAGRAPH.LEFT
                if ci < len(alignments):
                    if alignments[ci] == "center":
                        align = WD_ALIGN_PARAGRAPH.CENTER
                    elif alignments[ci] == "right":
                        align = WD_ALIGN_PARAGRAPH.RIGHT

                p = cell.paragraphs[0]
                p.alignment = align
                p.paragraph_format.space_before = Pt(2)
                p.paragraph_format.space_after  = Pt(2)
                _apply_inline(p, cell_text, base_size_pt=9 if ncols > 6 else 10, base_bold=(ri == 0))
                if ri == 0:
                    for run in p.runs:
                        run.font.color.rgb = WHITE
                        run.bold = True

        _set_col_widths(table, col_widths)
        # space after table
        self.doc.add_paragraph().paragraph_format.space_after = Pt(4)

    # ------------------------------------------------------------------
    # orchestrate
    # ------------------------------------------------------------------

    def build(self, blocks: List[_Block]) -> None:
        """Render all blocks into the document."""
        prev_heading_text = None

        for block in blocks:
            if block.kind == "blank":
                continue

            elif block.kind == "rule":
                self.render_rule()

            elif block.kind == "heading":
                text = block.text

                # remove duplicate adjacent section title:
                # if a ## heading is immediately followed by a # heading
                # with the same semantic content, skip the ##
                if text == prev_heading_text:
                    continue
                prev_heading_text = text
                self.render_heading(block.level, text)

            elif block.kind == "para":
                self.render_paragraph(block.text)
                prev_heading_text = None

            elif block.kind == "table":
                self.render_table(block.rows, block.alignments)
                prev_heading_text = None

            elif block.kind == "code":
                self.render_code(block.text)
                prev_heading_text = None


# ---------------------------------------------------------------------------
# Duplicate-heading suppression pre-pass
# ---------------------------------------------------------------------------

def _suppress_duplicate_headings(blocks: List[_Block]) -> List[_Block]:
    """
    If a ## heading text is immediately (with only blanks between)
    followed by a # heading with the same text, remove the ## heading.
    """
    result: List[_Block] = []
    i = 0
    while i < len(blocks):
        b = blocks[i]
        if b.kind == "heading":
            # look ahead past blanks for next heading
            j = i + 1
            while j < len(blocks) and blocks[j].kind == "blank":
                j += 1
            if (j < len(blocks) and
                    blocks[j].kind == "heading" and
                    blocks[j].text.strip() == b.text.strip()):
                # skip this one, keep the next (higher or equal level wins)
                i += 1
                continue
        result.append(b)
        i += 1
    return result


# ---------------------------------------------------------------------------
# manifest
# ---------------------------------------------------------------------------

def _compute_manifest(source_path: Path, output_path: Path,
                      blocks: List[_Block]) -> dict:
    source_text = source_path.read_text(encoding="utf-8")
    source_sha  = hashlib.sha256(source_text.encode()).hexdigest()
    output_sha  = (
        hashlib.sha256(output_path.read_bytes()).hexdigest()
        if output_path.exists() else "not-yet-written"
    )
    heading_count = sum(1 for b in blocks if b.kind == "heading")
    table_count   = sum(1 for b in blocks if b.kind == "table")
    para_count    = sum(1 for b in blocks if b.kind == "para")

    return {
        "source":       str(source_path),
        "source_sha256": source_sha,
        "output":       str(output_path),
        "output_sha256": output_sha,
        "headings":     heading_count,
        "tables":       table_count,
        "paragraphs":   para_count,
    }


# ---------------------------------------------------------------------------
# CLI entry point
# ---------------------------------------------------------------------------

def _build_arg_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        prog="render-reference-report.py",
        description=(
            "Convert a Selemene reference-report Markdown file into a "
            "polished DOCX (US Letter, Calibri, warm-maroon headings). "
            "No network access required. Output a manifest of headings/"
            "tables and source/output SHA-256 hashes."
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=textwrap.dedent(
            """\
            Examples
            --------
              # Render English individual report
              python3 render-reference-report.py \\
                --markdown output/gary-abitbol-2026-09-28/reports-12/individual-gary-abitbol-en.md \\
                --output /tmp/gary-abitbol-en.docx

              # Render French report with custom title
              python3 render-reference-report.py \\
                --markdown output/gary-abitbol-2026-09-28/reports-12/individual-gary-abitbol-fr.md \\
                --output /tmp/gary-abitbol-fr.docx \\
                --title "Gary Abitbol — Lecture intégrée du thème" \\
                --language fr

            Bundled runtime
            ---------------
              /Users/sheshnarayaniyer/.cache/codex-runtimes/\\
              codex-primary-runtime/dependencies/python/bin/python3 \\
              render-reference-report.py --markdown ... --output ...

            Manifest
            --------
              Prints a JSON manifest to stdout after writing the DOCX:
              heading count, table count, paragraph count,
              source SHA-256, output SHA-256, paths.

            Return codes
            ------------
              0  success
              1  input file not found or unreadable
              2  output directory does not exist
              3  python-docx write error
            """
        ),
    )
    p.add_argument(
        "--markdown", required=True, metavar="PATH",
        help="Path to the source Markdown file (.md).",
    )
    p.add_argument(
        "--output", required=True, metavar="PATH",
        help="Destination DOCX path (.docx).  Parent directory must exist.",
    )
    p.add_argument(
        "--title", default="", metavar="TEXT",
        help=(
            "Document title shown on the cover page. "
            "Defaults to the first H1 heading found in the Markdown, "
            "or the source filename stem."
        ),
    )
    p.add_argument(
        "--language", default="en", choices=["en", "fr"],
        help="Language label shown on the cover page (default: en).",
    )
    p.add_argument(
        "--no-toc", action="store_true",
        help="Suppress the static table of contents page.",
    )
    p.add_argument(
        "--no-cover", action="store_true",
        help="Suppress the cover page.",
    )
    p.add_argument(
        "--manifest-json", metavar="PATH",
        help="Write the JSON manifest to this file instead of stdout.",
    )
    return p


def main(argv: Optional[List[str]] = None) -> int:
    parser = _build_arg_parser()
    args = parser.parse_args(argv)

    # --- validate inputs ---
    md_path = Path(args.markdown)
    if not md_path.exists():
        print(f"[render-reference-report] ERROR: --markdown file not found: {md_path}",
              file=sys.stderr)
        return 1
    if not md_path.is_file():
        print(f"[render-reference-report] ERROR: --markdown is not a file: {md_path}",
              file=sys.stderr)
        return 1

    out_path = Path(args.output)
    if not out_path.parent.exists():
        print(f"[render-reference-report] ERROR: output directory does not exist: {out_path.parent}",
              file=sys.stderr)
        return 2

    # --- parse ---
    source_text = md_path.read_text(encoding="utf-8")
    blocks = _parse_markdown(source_text)
    blocks = _suppress_duplicate_headings(blocks)

    # derive title
    title = args.title.strip()
    if not title:
        for b in blocks:
            if b.kind == "heading" and b.level == 1:
                title = b.text
                break
        if not title:
            title = md_path.stem.replace("-", " ").replace("_", " ").title()

    # collect headings for TOC (H1 and H2 only)
    toc_headings: List[Tuple[int, str]] = [
        (b.level, b.text)
        for b in blocks
        if b.kind == "heading" and b.level <= 2
    ]

    # A chapter index remains consistent when source sections use different
    # Markdown heading depths. Detailed numbered subsections remain in-body.
    chapter_headings = [(level, text) for level, text in toc_headings
                        if re.match(r"^(?:Opening|Ouverture|Part(?:ie)?)\b", text)]
    if chapter_headings:
        toc_headings = [(1, text) for _, text in chapter_headings]

    # --- build ---
    builder = ReportBuilder(title=title, language=args.language)

    if not args.no_cover:
        builder.add_cover()

    if not args.no_toc and toc_headings:
        builder.add_toc(toc_headings)

    builder.build(blocks)

    # --- save ---
    try:
        builder.doc.save(str(out_path))
    except Exception as exc:
        print(f"[render-reference-report] ERROR saving DOCX: {exc}", file=sys.stderr)
        return 3

    # --- manifest ---
    manifest = _compute_manifest(md_path, out_path, blocks)
    manifest_json = json.dumps(manifest, indent=2)

    if args.manifest_json:
        mf_path = Path(args.manifest_json)
        mf_path.write_text(manifest_json + "\n", encoding="utf-8")
        print(f"[render-reference-report] manifest written to {mf_path}")
    else:
        print(manifest_json)

    print(f"[render-reference-report] OK  →  {out_path}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
