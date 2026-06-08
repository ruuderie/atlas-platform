#!/usr/bin/env python3
"""
Atlas Platform — Market Analysis Generator
==========================================
Reads the market_analysis_prompt.md system context and calls an LLM
(Anthropic Claude or Google Gemini) with a user-supplied target market block.

Usage:
  # Interactive mode — prompts you for market details:
  python3 scripts/market_analysis.py

  # One-liner with inline market description:
  python3 scripts/market_analysis.py --market "Trucking dispatch software in the US and Brazil"

  # Pass a file containing a detailed TARGET MARKET BLOCK:
  python3 scripts/market_analysis.py --market-file my_market_block.txt

  # Override filename (auto-derived from market description by default):
  python3 scripts/market_analysis.py --market "Healthcare SaaS in the US" --output-name healthcare_us

Environment variables (set ONE):
  ANTHROPIC_API_KEY   — uses Claude claude-opus-4-5 (or override with --model)
  GEMINI_API_KEY      — uses gemini-2.5-pro (or override with --model)

Output:
  Markdown → atlas-platform/docs/reports/market-analysis/<name>.md
  PDF      → atlas-platform/docs/reports/market-analysis/pdf/<name>_<date>.pdf
             (compiled automatically via the Rust LaTeX pipeline)
"""

import argparse
import json
import os
import re
import subprocess
import sys
import urllib.error
import urllib.parse
import urllib.request
from datetime import date
from pathlib import Path

# ---------------------------------------------------------------------------
# Paths (relative to the atlas-platform repo root)
# ---------------------------------------------------------------------------
SCRIPT_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parent          # atlas-platform/
PROMPT_FILE = REPO_ROOT / "docs/prompts/market_analysis_prompt.md"
REPORTS_DIR = REPO_ROOT / "docs/reports/market-analysis"
PDF_DIR = REPORTS_DIR / "pdf"
BACKEND_DIR = REPO_ROOT / "backend"


# ---------------------------------------------------------------------------
# Load the system prompt from the markdown file
# ---------------------------------------------------------------------------
def load_system_prompt() -> str:
    """
    Extract the SYSTEM CONTEXT block (between the ``` fences after
    '## SYSTEM CONTEXT') from the prompt file.
    """
    text = PROMPT_FILE.read_text(encoding="utf-8")

    # Find the section between ## SYSTEM CONTEXT and END OF SYSTEM CONTEXT
    match = re.search(
        r"## SYSTEM CONTEXT.*?```\n(.*?)```",
        text,
        re.DOTALL,
    )
    if not match:
        # Fallback: return everything between the two horizontal rules that
        # wrap the system context block, or just the entire file if needed.
        print("[warn] Could not isolate SYSTEM CONTEXT block — sending full prompt file.", file=sys.stderr)
        return text

    return match.group(1).strip()


# ---------------------------------------------------------------------------
# Derive a safe filename from the market description
# ---------------------------------------------------------------------------
def derive_filename(market: str) -> str:
    """
    Turn a free-text market description into a snake_case filename.
    e.g. "Trucking in the US and Brazil" -> "trucking_us_brazil"
    """
    # Country/region aliases
    aliases = {
        "united states": "us", "usa": "us", "u.s.": "us", "u.s.a.": "us",
        "brazil": "brazil", "brasil": "brazil",
        "haiti": "haiti",
        "united arab emirates": "uae", "uae": "uae",
        "europe": "europe", "eu": "europe",
        "canada": "canada",
        "mexico": "mexico",
        "global": "global", "worldwide": "global",
    }

    lower = market.lower()
    for long, short in aliases.items():
        lower = lower.replace(long, short)

    # Keep only alphanumeric and spaces
    lower = re.sub(r"[^a-z0-9 ]", " ", lower)
    # Collapse whitespace and replace with underscores
    lower = re.sub(r"\s+", "_", lower.strip())
    # Trim to a reasonable length
    lower = lower[:80].rstrip("_")
    return lower or "market_analysis"


# ---------------------------------------------------------------------------
# LLM API calls
# ---------------------------------------------------------------------------
def call_anthropic(system: str, user: str, model: str) -> str:
    api_key = os.environ["ANTHROPIC_API_KEY"]
    payload = {
        "model": model or "claude-opus-4-5",
        "max_tokens": 8192,
        "system": system,
        "messages": [{"role": "user", "content": user}],
    }
    req = urllib.request.Request(
        "https://api.anthropic.com/v1/messages",
        data=json.dumps(payload).encode(),
        headers={
            "x-api-key": api_key,
            "anthropic-version": "2023-06-01",
            "content-type": "application/json",
        },
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=300) as resp:
        data = json.loads(resp.read())
    return data["content"][0]["text"]


def call_gemini(system: str, user: str, model: str) -> str:
    api_key = os.environ["GEMINI_API_KEY"]
    model_id = model or "gemini-2.5-pro"
    url = (
        f"https://generativelanguage.googleapis.com/v1beta/models/"
        f"{model_id}:generateContent?key={api_key}"
    )
    payload = {
        "system_instruction": {"parts": [{"text": system}]},
        "contents": [{"role": "user", "parts": [{"text": user}]}],
        "generationConfig": {"maxOutputTokens": 8192},
    }
    req = urllib.request.Request(
        url,
        data=json.dumps(payload).encode(),
        headers={"content-type": "application/json"},
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=300) as resp:
        data = json.loads(resp.read())
    return data["candidates"][0]["content"]["parts"][0]["text"]


def call_llm(system: str, user: str, model: str | None) -> str:
    if "ANTHROPIC_API_KEY" in os.environ:
        print(f"  → Using Anthropic Claude ({model or 'claude-opus-4-5'})")
        return call_anthropic(system, user, model or "claude-opus-4-5")
    elif "GEMINI_API_KEY" in os.environ:
        print(f"  → Using Google Gemini ({model or 'gemini-2.5-pro'})")
        return call_gemini(system, user, model or "gemini-2.5-pro")
    else:
        print(
            "\n[ERROR] No API key found.\n"
            "Set ANTHROPIC_API_KEY or GEMINI_API_KEY in your environment.\n"
            "  export ANTHROPIC_API_KEY=sk-ant-...\n"
            "  export GEMINI_API_KEY=AIza...\n",
            file=sys.stderr,
        )
        sys.exit(1)


# ---------------------------------------------------------------------------
# PDF compilation via the Rust binary
# ---------------------------------------------------------------------------
def compile_pdf(md_path: Path, output_name: str) -> Path | None:
    today = date.today().isoformat()
    pdf_path = PDF_DIR / f"{output_name}_{today}.pdf"
    PDF_DIR.mkdir(parents=True, exist_ok=True)

    cmd = [
        "cargo", "run", "--bin", "generate_market_reports", "--",
        "--file", str(md_path.relative_to(BACKEND_DIR.parent)),
        "--output", str(pdf_path.relative_to(BACKEND_DIR.parent)),
    ]

    print(f"\n[PDF] Compiling PDF → {pdf_path.relative_to(REPO_ROOT)}")
    print(f"      Running: {' '.join(cmd)}")

    result = subprocess.run(
        cmd,
        cwd=str(BACKEND_DIR),
        capture_output=True,
        text=True,
    )

    if result.returncode != 0 and not pdf_path.exists():
        print(f"[warn] PDF compilation failed:\n{result.stderr}", file=sys.stderr)
        return None
    elif result.returncode != 0:
        print(f"[warn] pdflatex exited with non-zero code (expected) — PDF was produced.")

    if pdf_path.exists():
        size_kb = pdf_path.stat().st_size // 1024
        print(f"[PDF] ✓ PDF saved ({size_kb} KB): {pdf_path}")
        return pdf_path
    else:
        print("[warn] PDF file not found after compilation.", file=sys.stderr)
        return None


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
def main():
    parser = argparse.ArgumentParser(
        description="Generate an Atlas Platform market analysis report."
    )
    parser.add_argument(
        "--market", "-m",
        help="Free-text description of the target market/industry/geography.",
    )
    parser.add_argument(
        "--market-file", "-f",
        help="Path to a text file containing a detailed TARGET MARKET BLOCK.",
    )
    parser.add_argument(
        "--output-name", "-o",
        help="Override the auto-derived output filename (without extension).",
    )
    parser.add_argument(
        "--model",
        help="Override the default LLM model (e.g. claude-opus-4-5, gemini-2.5-pro).",
    )
    parser.add_argument(
        "--no-pdf",
        action="store_true",
        help="Skip the PDF compilation step.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print the assembled prompt without calling the LLM.",
    )
    args = parser.parse_args()

    # --- Collect market description ---
    if args.market_file:
        market_text = Path(args.market_file).read_text(encoding="utf-8").strip()
        output_name = args.output_name or derive_filename(Path(args.market_file).stem)
    elif args.market:
        market_text = args.market.strip()
        output_name = args.output_name or derive_filename(args.market)
    else:
        print("Atlas Platform — Market Analysis Generator")
        print("=" * 44)
        print(
            "\nDescribe the target market or industry you want analyzed.\n"
            "Be specific: include geography, customer type, company size, and competitors.\n"
            "Type your description below (press Enter twice when done):\n"
        )
        lines = []
        try:
            while True:
                line = input()
                if not line and lines and not lines[-1]:
                    break
                lines.append(line)
        except EOFError:
            pass
        market_text = "\n".join(lines).strip()
        if not market_text:
            print("[ERROR] No market description provided.", file=sys.stderr)
            sys.exit(1)
        output_name = args.output_name or derive_filename(market_text)

    # --- Build the user message (TARGET MARKET BLOCK) ---
    user_message = f"""TARGET MARKET BLOCK:

{market_text}

Please produce the full structured strategic analysis report as specified in the ANALYSIS INSTRUCTIONS above.

IMPORTANT — LaTeX-Friendly Output Rules (for PDF compilation):
- Use ASCII math: <=, >=, !=, ~ (NOT Unicode: ≤ ≥ ≠ ≈)
- Use ASCII arrows: ->, <-, <-> (NOT Unicode: → ← ↔)
- In code blocks, use only standard ASCII box-drawing: |, -, + (NOT Unicode: │ ├── └──)
- Use standard Markdown table syntax with clear headers
- No emojis or accented characters inside code blocks
"""

    # --- Load system prompt ---
    print(f"\n[1/4] Loading system context from {PROMPT_FILE.relative_to(REPO_ROOT)}")
    system_prompt = load_system_prompt()
    print(f"      System context: {len(system_prompt):,} characters")

    # --- Dry run ---
    if args.dry_run:
        print("\n--- SYSTEM PROMPT ---")
        print(system_prompt[:500] + "..." if len(system_prompt) > 500 else system_prompt)
        print("\n--- USER MESSAGE ---")
        print(user_message)
        print(f"\n[dry-run] Output would be saved as: {output_name}.md")
        return

    # --- Call LLM ---
    print(f"\n[2/4] Calling LLM for market: {market_text[:80]}...")
    try:
        report_md = call_llm(system_prompt, user_message, args.model)
    except urllib.error.HTTPError as e:
        body = e.read().decode(errors="replace")
        print(f"[ERROR] HTTP {e.code}: {body}", file=sys.stderr)
        sys.exit(1)

    # --- Save markdown ---
    REPORTS_DIR.mkdir(parents=True, exist_ok=True)
    md_path = REPORTS_DIR / f"{output_name}.md"

    # If file already exists, version it
    if md_path.exists():
        today = date.today().isoformat()
        md_path = REPORTS_DIR / f"{output_name}_{today}.md"

    print(f"\n[3/4] Saving markdown report → {md_path.relative_to(REPO_ROOT)}")
    md_path.write_text(report_md, encoding="utf-8")
    size_kb = md_path.stat().st_size // 1024
    print(f"      ✓ Saved ({size_kb} KB, {len(report_md):,} characters)")

    # --- Compile PDF ---
    if not args.no_pdf:
        print(f"\n[4/4] Compiling PDF...")
        compile_pdf(md_path, output_name)
    else:
        print("\n[4/4] PDF compilation skipped (--no-pdf).")

    print(f"\n✓ Done. Report: {md_path}")


if __name__ == "__main__":
    main()
