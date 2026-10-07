#!/usr/bin/env python3
"""Reproduce the reviewed Legge1882 excerpts from pinned Princeton OCR XML.

No network requests. Never guesses missing content. Curated spans were checked
against the ORIGINAL scan, not copied from a modern prepared transcription.
Requires only Python3 standard library. See i-ching-transcription-notes.md.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import xml.etree.ElementTree as ET

ITEM = "sacredbooksofchi16conf"
SOURCE_URL = "https://archive.org/details/" + ITEM
XML_SHA256 = "3e9823f65f22f7679277128649fcc4d1ef53ccf97954d8cb3691e57841022f5b"
SCANDATA_SHA256 = "f3951fa3da2df98af408d2495d867c125cdd287ef7195210c340d8d13ef16ea3"
PDF_SHA256 = "216991696663472339e90f1a64aad45998fbe9de711885ab00744572475896c6"
REVIEW_DATE = "2026-10-07"
STATUS = "scan_reviewed_normalized_transcription"
# Each span is [zero-based PDF/object page, zero-based paragraph, line start,
# line end]. None means the paragraph boundary. End is exclusive.
SPANS = {1: {'judgment': [[90, 4, 1, None]], 'image': [[300, 3, None, None]]},
 2: {'judgment': [[92, 2, None, None], [93, 2, None, None]], 'image': [[301, 1, None, None]]},
 3: {'judgment': [[95, 2, None, None]], 'image': [[303, 1, None, None]]},
 4: {'judgment': [[97, 2, None, None], [98, 1, None, None]], 'image': [[304, 1, None, None]]},
 5: {'judgment': [[100, 2, None, None]], 'image': [[305, 2, None, None], [306, 1, None, None]]},
 6: {'judgment': [[102, 2, None, None]], 'image': [[307, 1, None, None]]},
 7: {'judgment': [[104, 2, None, None], [105, 2, None, None]], 'image': [[308, 5, None, None]]},
 8: {'judgment': [[106, 2, None, None], [107, 2, None, None]], 'image': [[310, 1, None, None]]},
 9: {'judgment': [[109, 2, None, None]], 'image': [[311, 3, None, None]]},
 10: {'judgment': [[111, 2, None, None]], 'image': [[313, 1, None, None]]},
 11: {'judgment': [[114, 2, None, None]], 'image': [[314, 3, None, None]]},
 12: {'judgment': [[116, 2, None, None], [117, 1, None, None]], 'image': [[315, 4, None, None]]},
 13: {'judgment': [[119, 2, None, None]], 'image': [[317, 1, None, None]]},
 14: {'judgment': [[121, 3, None, None]], 'image': [[318, 1, None, None]]},
 15: {'judgment': [[122, 2, None, None]], 'image': [[319, 2, None, None]]},
 16: {'judgment': [[124, 2, None, None]], 'image': [[320, 2, None, None], [321, 1, None, None]]},
 17: {'judgment': [[126, 2, None, None]], 'image': [[322, 2, None, None]]},
 18: {'judgment': [[128, 2, None, None]], 'image': [[323, 2, None, None]]},
 19: {'judgment': [[130, 3, None, None]], 'image': [[324, 2, None, None]]},
 20: {'judgment': [[132, 2, None, None], [133, 2, None, None]], 'image': [[325, 4, None, None]]},
 21: {'judgment': [[134, 2, None, None]], 'image': [[326, 6, None, None]]},
 22: {'judgment': [[136, 3, None, None]], 'image': [[327, 6, None, None]]},
 23: {'judgment': [[138, 2, None, None]], 'image': [[329, 1, None, None]]},
 24: {'judgment': [[140, 2, None, None], [141, 1, None, None]], 'image': [[330, 1, None, None]]},
 25: {'judgment': [[142, 2, None, None], [143, 1, None, None]], 'image': [[332, 1, None, None]]},
 26: {'judgment': [[145, 2, None, None]], 'image': [[333, 1, None, None]]},
 27: {'judgment': [[147, 3, None, None]], 'image': [[334, 1, None, None]]},
 28: {'judgment': [[149, 3, None, None]], 'image': [[335, 2, None, None]]},
 29: {'judgment': [[151, 3, None, None]], 'image': [[336, 3, None, None]]},
 30: {'judgment': [[153, 3, None, None], [154, 1, None, None]], 'image': [[337, 4, None, None]]},
 31: {'judgment': [[156, 2, None, None]], 'image': [[338, 4, None, None]]},
 32: {'judgment': [[158, 2, None, None]], 'image': [[340, 1, None, None]]},
 33: {'judgment': [[160, 2, None, None]], 'image': [[341, 1, None, None]]},
 34: {'judgment': [[162, 2, None, None]], 'image': [[342, 1, None, None]]},
 35: {'judgment': [[164, 2, None, None]], 'image': [[343, 1, None, None]]},
 36: {'judgment': [[167, 3, None, None], [168, 1, None, None]], 'image': [[344, 1, None, None]]},
 37: {'judgment': [[169, 4, None, None], [170, 1, None, None], [170, 2, None, None]],
      'image': [[345, 2, None, None]]},
 38: {'judgment': [[172, 2, None, None]], 'image': [[347, 1, None, None]]},
 39: {'judgment': [[174, 3, None, None], [175, 1, None, None]], 'image': [[348, 1, None, None]]},
 40: {'judgment': [[177, 2, None, None]], 'image': [[349, 2, None, None]]},
 41: {'judgment': [[179, 2, None, None], [180, 1, None, None]], 'image': [[350, 4, None, None]]},
 42: {'judgment': [[182, 2, None, None]], 'image': [[352, 1, None, None]]},
 43: {'judgment': [[184, 2, None, None], [185, 1, None, None]], 'image': [[353, 1, None, None]]},
 44: {'judgment': [[187, 2, None, None]], 'image': [[354, 3, None, None]]},
 45: {'judgment': [[189, 2, None, None], [190, 1, None, None]], 'image': [[356, 1, None, None]]},
 46: {'judgment': [[192, 2, None, None], [193, 2, None, None]], 'image': [[357, 2, None, None]]},
 47: {'judgment': [[194, 2, None, None], [195, 2, None, None]], 'image': [[358, 3, None, None]]},
 48: {'judgment': [[197, 3, None, None], [198, 1, None, None]], 'image': [[360, 1, None, None]]},
 49: {'judgment': [[200, 2, None, None]], 'image': [[361, 2, None, None]]},
 50: {'judgment': [[202, 2, None, None]], 'image': [[362, 3, None, None]]},
 51: {'judgment': [[205, 2, None, None], [206, 1, None, None]], 'image': [[363, 5, None, None]]},
 52: {'judgment': [[208, 2, None, None], [209, 2, None, None]],
      'image': [[364, 9, None, None], [365, 1, None, None]]},
 53: {'judgment': [[211, 3, None, None]], 'image': [[366, 1, None, None]]},
 54: {'judgment': [[213, 2, None, None], [213, 3, None, None]], 'image': [[367, 1, None, None]]},
 55: {'judgment': [[216, 3, None, None], [217, 1, None, None]], 'image': [[368, 6, None, None]]},
 56: {'judgment': [[220, 2, None, None]], 'image': [[370, 2, None, None]]},
 57: {'judgment': [[222, 2, None, None], [223, 2, None, None]], 'image': [[371, 2, None, None]]},
 58: {'judgment': [[225, 2, None, None]], 'image': [[373, 1, None, None]]},
 59: {'judgment': [[227, 2, None, None], [228, 1, None, None]], 'image': [[374, 1, None, None]]},
 60: {'judgment': [[230, 3, None, None]], 'image': [[375, 1, None, None]]},
 61: {'judgment': [[232, 2, None, 2], [233, 2, None, None]], 'image': [[376, 3, None, None]]},
 62: {'judgment': [[234, 2, None, None], [235, 1, None, None]], 'image': [[377, 4, None, None]]},
 63: {'judgment': [[237, 3, None, None], [238, 1, None, None]],
      'image': [[378, 7, None, None], [379, 1, None, None]]},
 64: {'judgment': [[240, 2, None, None]], 'image': [[379, 9, None, None]]}}
# Every substitution is scan-confirmed and retained in the correction ledger.
# Historical italic K/tone marks and the z-shaped consonant use ASCII below.
CORRECTIONS = {1: {'judgment': [('KM en', 'Khien')]},
 3: {'image': [('A"un', 'Kun')]},
 9: {'judgment': [('Kkii', 'Khu')], 'image': [('AV211', 'Khu')]},
 12: {'judgment': [('Pht', 'Phi')], 'image': [('(the manifestation) of)', '(the manifestation) of')]},
 13: {'judgment': [('advantagfeous', 'advantageous')]},
 14: {'image': [('Ta Yd', 'Ta Yu')]},
 15: {'judgment': [('K/t ien', 'Khien')], 'image': [('Kk\\er\\', 'Khien'), ('accordine', 'according')]},
 16: {'judgment': [('Yii', 'Yu')]},
 17: {'image': [('trig-ram', 'trigram')]},
 26: {'judgment': [('Ta Khii', 'Ta Khu')], 'image': [('Ta Khu..', 'Ta Khu.')]},
 27: {'judgment': [('1 indicates', 'I indicates')]},
 28: {'image': [('T a. Kwo', 'Ta Kwo')]},
 34: {'judgment': [('Ti Awang', 'Ta Kwang')], 'image': [('Ta Tsfwang', 'Ta Kwang')]},
 35: {'judgment': [('3in', 'Zin')], 'image': [('3in-', 'Zin.')]},
 36: {'judgment': [('Mi no- 1', 'Ming I')], 'image': [('Ming 1', 'Ming I')]},
 37: {'judgment': [("A'ia Zan", 'Kia Zan')], 'image': [('Afia. Zan', 'Kia Zan')]},
 39: {'judgment': [('A^ien', 'Kien')], 'image': [('A^ien', 'Kien')]},
 40: {'judgment': [('A"ieh', 'Kieh')], 'image': [('K\\ eh', 'Kieh')]},
 42: {'image': [('Yl.', 'Yi.')]},
 45: {'judgment': [('3hui', 'Zhui')], 'image': [('3hui', 'Zhui')]},
 48: {'judgment': [('3ing', 'Zing')], 'image': [('3ing', 'Zing')]},
 51: {'judgment': [("A'an", 'Kan')], 'image': [("A'an", 'Kan')]},
 56: {'judgment': [('Lii', 'Lu')], 'image': [('Lii', 'Lu')]},
 60: {'judgment': [('A^ieh', 'Kieh')], 'image': [("A'ieh", 'Kieh')]},
 61: {'judgment': [('Kur\\g', 'Kung')], 'image': [('A^ung FCi', 'Kung Fu')]},
 63: {'judgment': [('K\\ 3 1', 'Ki Zi')], 'image': [('K\\ 3i', 'Ki Zi')]},
 64: {'judgment': [('Wei 31', 'Wei Zi')], 'image': [('Wei 3i', 'Wei Zi')]}}


def normalize(raw):
    # Joining print end-line word splits is mechanical, not paraphrasing.
    raw = re.sub(r"(?<=\w)-[ \t]*\n[ \t]*(?=\w)", "", raw)
    return re.sub(r"\s+", " ", raw).strip()


def checked_bytes(path, expected):
    data = Path(path).read_bytes()
    actual = hashlib.sha256(data).hexdigest()
    if actual != expected:
        raise ValueError(f"Pinned source hash mismatch for {Path(path).name}")
    return data


def roman_number(raw):
    raw = raw.replace(" ", "").replace("1", "I")
    values = {"I": 1, "V": 5, "X": 10, "L": 50}
    return sum((-1 if i + 1 < len(raw) and values[c] < values[raw[i+1]]
                else 1) * values[c] for i, c in enumerate(raw))


def extract(xml_path, scandata_path):
    xml = checked_bytes(xml_path, XML_SHA256)
    scan = checked_bytes(scandata_path, SCANDATA_SHA256)
    objects = ET.fromstring(xml).findall(".//OBJECT")
    access = [p for p in ET.fromstring(scan).findall(".//pageData/page")
              if p.findtext("addToAccessFormats") == "true"]
    if len(objects) != 524 or len(access) != 524:
        raise ValueError("Unexpected original scan/access page inventory")
    source = {"title": "The Sacred Books of China, Part II: The Yi King",
              "translator": "James Legge", "year": 1882,
              "edition": "Oxford, Clarendon Press; Sacred Books of the East, volume XVI",
              "url": SOURCE_URL, "scanContributor": "Princeton Theological Seminary Library",
              "transcriptionStatus": STATUS}
    normalizations = [
        "Whitespace collapsed; print end-line word splits joined.",
        "Roman section markers and editorial headings excluded.",
        "Historical transliteration tone marks/italic consonant typography normalized to ASCII; the z-shaped consonant is rendered Z.",
        "English wording, archaic grammar and source punctuation retained; no modern paraphrases added.",
    ]
    metadata = dict(source, schemaVersion=1, retrievedOn=REVIEW_DATE,
                    totalHexagrams=64, coverage=["64 primary judgments", "64 Great Symbolism images"],
                    exclusions=["six line texts", "small Symbolism/line commentary", "modern meanings", "translator footnotes"],
                    normalizations=normalizations,
                    uncertainty="OCR is not authoritative. All selected passages were visually checked against the pinned original PDF; this normalized transcription is not a character-perfect diplomatic edition or an independent second proofread.",
                    rightsEvidence={"archiveStatus": "NOT_IN_COPYRIGHT", "region": "US",
                                   "sourcePublication": 1882,
                                   "scope": "Original historical text; source/publication facts, not a publisher licensing attestation."})
    records, audits = [], []
    for number in range(1, 65):
        if number not in SPANS:
            raise ValueError(f"Missing curated ID {number}")
        record = {"number": number}
        provenance = {}
        notes = ["Original scan wording reviewed; transliteration typography normalized as metadata documents."]
        audit = {"number": number, "transcriptionStatus": STATUS, "fields": {}}
        for field in ("judgment", "image"):
            chunks, segments = [], []
            for object_index, paragraph_index, start, end in SPANS[number][field]:
                if not 90 <= object_index <= 380:
                    raise ValueError("Curated span outside approved text/Symbolism pages")
                obj = objects[object_index]
                para = obj.findall(".//PARAGRAPH")[paragraph_index]
                lines = [" ".join("".join(w.itertext()) for w in line.findall(".//WORD"))
                         for line in para.findall(".//LINE")]
                chosen = lines[start:end]
                if not chosen:
                    raise ValueError(f"Empty span for ID {number} {field}")
                chunks.append("\n".join(chosen))
                leaf = int(access[object_index].attrib["leafNum"])
                segments.append({"printedPage": object_index - 33,
                                 "pdfPage": object_index + 1,
                                 "archiveLeaf": leaf, "paragraphIndex": paragraph_index,
                                 "lineStart": start, "lineEndExclusive": end,
                                 "url": SOURCE_URL + f"/page/n{leaf}/mode/1up"})
            raw = "\n".join(chunks)
            if field == "image":
                marker = re.match(r"^\s*([IVXL 1]+)\.\s+", raw)
                if not marker or roman_number(marker.group(1)) != number:
                    raise ValueError(f"Great Symbolism ID marker mismatch {number}")
                raw_without_marker = raw[marker.end():]
            else:
                raw_without_marker = raw
            value = normalize(raw_without_marker)
            edits = []
            for before, after in CORRECTIONS.get(number, {}).get(field, []):
                if value.count(before) != 1:
                    raise ValueError(f"Correction anchor mismatch ID {number} {field}: {before}")
                value = value.replace(before, after, 1)
                edits.append({"ocr": before, "transcribed": after,
                              "basis": "Visually checked against original printed source at listed pages."})
            if not value or len(value) < 40 or len(value) > 700:
                raise ValueError(f"Unexpected passage length for ID {number} {field}")
            if re.search(r"(?:\\|\^|Paragraph \d|Line \d|Hexagram \d|\b[2-6]\.\s)", value):
                raise ValueError(f"Unresolved OCR/commentary marker in ID {number} {field}")
            record[field] = value
            pages = list(dict.fromkeys(segment["printedPage"] for segment in segments))
            provenance[field + "Page"] = pages[0]
            provenance[field + "Pages"] = pages
            provenance[field + "Sources"] = segments
            if edits:
                notes.append(f"{field}: {len(edits)} explicit OCR/transliteration corrections recorded in source provenance.")
            audit["fields"][field] = {"rawOcr": raw,
                                       "normalizedOcrBeforeCorrections": normalize(raw_without_marker),
                                       "reviewedText": value, "sourceSpans": segments,
                                       "corrections": edits}
        if number == 13:
            notes.append("Great Symbolism retains the original printed unmatched ')' after 'this' on page284.")
        record.update(provenance=provenance, transcriptionStatus=STATUS, transcriptionNotes=notes)
        records.append(record)
        audits.append(audit)
    if len({r["judgment"] for r in records}) != 64 or len({r["image"] for r in records}) != 64:
        raise ValueError("Unexpected duplicate excerpts")
    data = {"metadata": metadata, "hexagrams": records}
    audit_data = {"schemaVersion": 1, "preparedOn": REVIEW_DATE, "source": source,
                  "sourceFiles": [
                      {"url": "https://archive.org/download/" + ITEM + "/" + ITEM + "_djvu.xml", "sha256": XML_SHA256},
                      {"url": "https://archive.org/download/" + ITEM + "/" + ITEM + "_scandata.xml", "sha256": SCANDATA_SHA256},
                      {"url": "https://archive.org/download/" + ITEM + "/" + ITEM + ".pdf", "sha256": PDF_SHA256}],
                  "coverage": {"ids": list(range(1, 65)), "judgments": 64, "greatSymbolismImages": 64,
                               "movingLineTexts": 0, "unresolvedSemanticOcrIssuesFound": 0,
                               "independentSecondProofread": False},
                  "normalizations": normalizations, "records": audits}
    return data, audit_data


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--xml", required=True)
    parser.add_argument("--scandata", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--provenance", required=True)
    args = parser.parse_args()
    data, audit = extract(args.xml, args.scandata)
    Path(args.output).write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n")
    dataset_sha = hashlib.sha256(Path(args.output).read_bytes()).hexdigest()
    audit["datasetSha256"] = dataset_sha
    Path(args.provenance).write_text(json.dumps(audit, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"hexagrams": 64, "judgments": 64, "images": 64,
                      "uniqueJudgments": len({r["judgment"] for r in data["hexagrams"]}),
                      "uniqueImages": len({r["image"] for r in data["hexagrams"]}),
                      "corrections": sum(len(f["corrections"]) for r in audit["records"] for f in r["fields"].values()),
                      "datasetSha256": dataset_sha, "status": STATUS}))


if __name__ == "__main__":
    main()
