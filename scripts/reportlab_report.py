#!/usr/bin/env python3
"""Generate a minimal Cerberus security audit PDF with ReportLab."""

from __future__ import annotations

import argparse
import json
from html import escape
from pathlib import Path
from typing import Any

from reportlab.lib import colors
from reportlab.lib.enums import TA_RIGHT
from reportlab.lib.pagesizes import LETTER
from reportlab.lib.styles import ParagraphStyle, getSampleStyleSheet
from reportlab.lib.units import inch
from reportlab.platypus import (
    HRFlowable,
    Paragraph,
    SimpleDocTemplate,
    Spacer,
    Table,
    TableStyle,
)


def load_json(path: Path, default: Any) -> Any:
    if not path.exists():
        return default
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def text(value: Any) -> str:
    if value is None:
        return ""
    return escape(str(value))


def label(value: Any) -> str:
    value = str(value or "").replace("_", " ").strip()
    return value[:1].upper() + value[1:] if value else ""


def count_status(findings: list[dict[str, Any]], status: str) -> int:
    return sum(1 for finding in findings if str(finding.get("status", "")).lower() == status)


def build_pdf(mission_path: Path, findings_path: Path, evidence_path: Path, out: Path) -> None:
    mission = load_json(mission_path, {})
    findings = load_json(findings_path, [])
    evidence = load_json(evidence_path, [])

    out.parent.mkdir(parents=True, exist_ok=True)

    doc = SimpleDocTemplate(
        str(out),
        pagesize=LETTER,
        rightMargin=0.72 * inch,
        leftMargin=0.72 * inch,
        topMargin=0.64 * inch,
        bottomMargin=0.64 * inch,
        title="Cerberus Security Audit Report",
        author="Araskova Labs",
    )

    styles = getSampleStyleSheet()
    styles.add(
        ParagraphStyle(
            name="Meta",
            parent=styles["Normal"],
            fontName="Helvetica",
            fontSize=8.5,
            leading=12,
            textColor=colors.HexColor("#475569"),
        )
    )
    styles.add(
        ParagraphStyle(
            name="RightMeta",
            parent=styles["Meta"],
            alignment=TA_RIGHT,
        )
    )
    styles.add(
        ParagraphStyle(
            name="FindingTitle",
            parent=styles["Heading2"],
            fontName="Helvetica-Bold",
            fontSize=13,
            leading=16,
            spaceBefore=14,
            spaceAfter=6,
            textColor=colors.HexColor("#111827"),
        )
    )
    styles.add(
        ParagraphStyle(
            name="SmallHeading",
            parent=styles["Heading3"],
            fontName="Helvetica-Bold",
            fontSize=9,
            leading=12,
            spaceBefore=8,
            spaceAfter=3,
            textColor=colors.HexColor("#334155"),
        )
    )

    story = []
    story.append(Paragraph("Cerberus Security Audit Report", styles["Title"]))
    story.append(Paragraph("Governed AI-assisted security audit", styles["Meta"]))
    story.append(Spacer(1, 0.16 * inch))
    story.append(HRFlowable(width="100%", thickness=0.8, color=colors.HexColor("#CBD5E1")))
    story.append(Spacer(1, 0.18 * inch))

    meta_data = [
        ["Mission ID", text(mission.get("id"))],
        ["Objective", text(mission.get("objective"))],
        ["Scope Root", text(mission.get("scope_root", "."))],
        ["Created", text(mission.get("created_at"))],
    ]
    story.append(clean_table(meta_data, [1.25 * inch, 5.05 * inch]))
    story.append(Spacer(1, 0.22 * inch))

    summary_data = [
        ["Findings", str(len(findings))],
        ["Open", str(count_status(findings, "open"))],
        ["Verified", str(count_status(findings, "verified"))],
        ["Evidence Records", str(len(evidence))],
    ]
    story.append(Paragraph("Summary", styles["Heading2"]))
    story.append(clean_table(summary_data, [1.55 * inch, 4.75 * inch]))

    story.append(Spacer(1, 0.18 * inch))
    story.append(Paragraph("Findings", styles["Heading2"]))
    if not findings:
        story.append(Paragraph("No findings recorded.", styles["Normal"]))

    for index, finding in enumerate(findings, start=1):
        story.append(
            Paragraph(f"{index}. {text(finding.get('title', 'Untitled finding'))}", styles["FindingTitle"])
        )
        detail_data = [
            ["ID", text(finding.get("id"))],
            ["Severity", label(finding.get("severity"))],
            ["Status", label(finding.get("status"))],
            ["Evidence IDs", text(", ".join(finding.get("evidence_ids", [])))],
        ]
        story.append(clean_table(detail_data, [1.2 * inch, 5.1 * inch]))
        story.append(Paragraph("Evidence", styles["SmallHeading"]))
        story.append(Paragraph(text(finding.get("evidence", "No evidence text recorded.")), styles["BodyText"]))
        story.append(Paragraph("Remediation", styles["SmallHeading"]))
        story.append(
            Paragraph(text(finding.get("remediation", "No remediation recorded.")), styles["BodyText"])
        )
        story.append(Spacer(1, 0.1 * inch))

    doc.build(story)


def clean_table(data: list[list[str]], widths: list[float]) -> Table:
    rows = [[Paragraph(text(key), getSampleStyleSheet()["BodyText"]), Paragraph(text(value), getSampleStyleSheet()["BodyText"])] for key, value in data]
    table = Table(rows, colWidths=widths, hAlign="LEFT")
    table.setStyle(
        TableStyle(
            [
                ("FONTNAME", (0, 0), (0, -1), "Helvetica-Bold"),
                ("FONTSIZE", (0, 0), (-1, -1), 8.7),
                ("TEXTCOLOR", (0, 0), (0, -1), colors.HexColor("#334155")),
                ("TEXTCOLOR", (1, 0), (1, -1), colors.HexColor("#111827")),
                ("VALIGN", (0, 0), (-1, -1), "TOP"),
                ("LINEBELOW", (0, 0), (-1, -1), 0.25, colors.HexColor("#E2E8F0")),
                ("BOTTOMPADDING", (0, 0), (-1, -1), 6),
                ("TOPPADDING", (0, 0), (-1, -1), 6),
                ("LEFTPADDING", (0, 0), (-1, -1), 0),
                ("RIGHTPADDING", (0, 0), (-1, -1), 8),
            ]
        )
    )
    return table


def main() -> None:
    parser = argparse.ArgumentParser(description="Generate a Cerberus PDF report with ReportLab.")
    parser.add_argument("--mission", required=True, type=Path)
    parser.add_argument("--findings", required=True, type=Path)
    parser.add_argument("--evidence", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    build_pdf(args.mission, args.findings, args.evidence, args.out)


if __name__ == "__main__":
    main()
