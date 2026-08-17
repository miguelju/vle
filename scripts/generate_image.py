#!/usr/bin/env python
"""Generate an illustration with OpenAI's image API and record how it was made.

The sibling of [`second_opinion.py`](second_opinion.py): same 1Password
credential, same stdlib-only constraint (``urllib`` — no ``openai`` package, so
nothing is installed into the ``vle`` conda environment), same insistence on
writing down the provenance of anything an external model produced.

Usage (from the repo root)::

    ~/miniconda3/envs/vle/bin/python scripts/generate_image.py \\
        scripts/prompts/distillation-bases.md \\
        --out docs/assets/distillation-bases.png \\
        --size 1536x1024 --quality high

Alongside ``<out>.png`` it writes ``<out>.json``: the model, the exact prompt,
the size and quality, the timestamp, the token usage, the computed cost, and —
when the API returns one — the *revised* prompt the model actually rendered.
That last field matters. These models rewrite prompts internally, so the text
you sent is not necessarily the text that produced the picture, and without the
revision on record a regenerated image is unexplainable.

## What model, and why

``gpt-image-2`` is the current OpenAI image model (default snapshot
``gpt-image-2-2026-04-21``). DALL·E 3 was retired on 2026-03-04 and is not an
option. The GPT-Image family is autoregressive rather than diffusion-based,
which is why it renders legible text far better than DALL·E did — the reason it
is usable at all for a labelled technical figure.

**It is still not a plotting library.** Nothing here guarantees that a curve is
quantitatively right, that four labels attach to the four things they name, or
that an axis means anything. Use this for *conceptual* illustration — an
apparatus, a metaphor, a hero image — and use matplotlib for anything a reader
is meant to read numbers off. Every image produced by this script should be
looked at by a human before it lands in a document.

## Cost

Billing is token-based, not per-image: ``$5.00`` per million text input tokens
and ``$30.00`` per million image output tokens. In practice a 1536x1024 render
is roughly ``$0.01`` at ``--quality low``, ``$0.08`` at ``medium`` and ``$0.32``
at ``high``. The script prints the actual usage-derived cost when the API
reports usage.

## References

- Image generation guide: https://developers.openai.com/api/docs/guides/image-generation
- Model card: https://developers.openai.com/api/docs/models/gpt-image-2
"""

from __future__ import annotations

import argparse
import base64
import json
import sys
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from _openai_key import read_key  # noqa: E402

ENDPOINT = "https://api.openai.com/v1/images/generations"

#: USD per million tokens: (text input, image input, image output).
PRICING = {
    "gpt-image-2": (5.00, 8.00, 30.00),
    "gpt-image-1": (5.00, 10.00, 40.00),
}

#: gpt-image-2 accepts arbitrary sizes within these bounds. Checked locally so a
#: typo costs a millisecond instead of a round trip and an opaque 400.
MAX_EDGE = 3840
MIN_PIXELS = 655_360
MAX_PIXELS = 8_294_400


def parse_size(size: str) -> str:
    """Validate a ``WIDTHxHEIGHT`` string against gpt-image-2's constraints.

    ``"auto"`` passes through untouched — the model picks.
    """
    if size == "auto":
        return size
    try:
        w, h = (int(v) for v in size.lower().split("x"))
    except ValueError:
        raise SystemExit(f"--size must be WIDTHxHEIGHT or 'auto', got {size!r}") from None
    problems = []
    if w % 16 or h % 16:
        problems.append("both edges must be multiples of 16")
    if max(w, h) > MAX_EDGE:
        problems.append(f"the long edge must be <= {MAX_EDGE} px")
    if not (1 / 3) <= w / h <= 3:
        problems.append("the aspect ratio must be between 1:3 and 3:1")
    if not MIN_PIXELS <= w * h <= MAX_PIXELS:
        problems.append(f"total pixels must be {MIN_PIXELS:,}-{MAX_PIXELS:,} (this is {w * h:,})")
    if problems:
        raise SystemExit(f"--size {size} is not valid: " + "; ".join(problems))
    return f"{w}x{h}"


def generate(key: str, body: dict) -> dict:
    req = urllib.request.Request(
        ENDPOINT,
        data=json.dumps(body).encode(),
        headers={"Authorization": f"Bearer {key}", "Content-Type": "application/json"},
        method="POST",
    )
    try:
        # Generous timeout: a high-quality render is a slow synchronous call.
        with urllib.request.urlopen(req, timeout=900) as resp:
            return json.load(resp)
    except urllib.error.HTTPError as exc:
        sys.exit(f"HTTP {exc.code}: {exc.read().decode()[:2000]}")


def image_bytes(item: dict) -> bytes:
    """Pull the raw image out of one `data[]` entry.

    The GPT-Image models always return base64 rather than a URL, but the `url`
    branch is kept so this keeps working if that ever changes or an older model
    is passed with ``--model``.
    """
    if item.get("b64_json"):
        return base64.b64decode(item["b64_json"])
    if item.get("url"):
        with urllib.request.urlopen(item["url"], timeout=300) as resp:
            return resp.read()
    sys.exit(f"response entry carried neither b64_json nor url: {sorted(item)}")


def main() -> None:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("prompt", help="path to a file containing the prompt")
    ap.add_argument("--out", required=True, help="output image path (.png/.jpg/.webp)")
    ap.add_argument("--model", default="gpt-image-2")
    ap.add_argument("--size", default="1536x1024", help="WIDTHxHEIGHT or 'auto'")
    ap.add_argument("--quality", default="high", choices=["low", "medium", "high", "auto"])
    ap.add_argument("--n", type=int, default=1, help="how many variants to generate")
    ap.add_argument(
        "--background", default="auto", choices=["auto", "transparent", "opaque"],
        help="gpt-image-2 does not support transparent backgrounds",
    )
    ap.add_argument(
        "--moderation", default="auto", choices=["auto", "low"],
        help="'low' relaxes the content filter; leave on auto unless it misfires",
    )
    args = ap.parse_args()

    prompt = Path(args.prompt).read_text().strip()
    if not prompt:
        sys.exit(f"{args.prompt} is empty")
    size = parse_size(args.size)
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    fmt = out.suffix.lstrip(".").lower()
    if fmt == "jpg":
        fmt = "jpeg"
    if fmt not in {"png", "jpeg", "webp"}:
        sys.exit(f"--out must end in .png, .jpg/.jpeg or .webp, got {out.suffix!r}")

    body = {
        "model": args.model,
        "prompt": prompt,
        "size": size,
        "quality": args.quality,
        "n": args.n,
        "background": args.background,
        "moderation": args.moderation,
        "output_format": fmt,
    }

    print(f"prompt: {len(prompt):,} chars -> {args.model} "
          f"({size}, quality={args.quality}, n={args.n})")
    payload = generate(read_key(), body)

    data = payload.get("data") or []
    if not data:
        sys.exit(f"no images in response: {json.dumps(payload)[:800]}")

    written = []
    for i, item in enumerate(data):
        # One image keeps the requested name; variants get -1, -2, ... suffixes.
        path = out if len(data) == 1 else out.with_stem(f"{out.stem}-{i + 1}")
        path.write_bytes(image_bytes(item))
        written.append(path)
        print(f"wrote {path} ({path.stat().st_size / 1024:.0f} KB)")

    usage = payload.get("usage") or {}
    tin = usage.get("input_tokens", 0)
    tout = usage.get("output_tokens", 0)
    text_in = (usage.get("input_tokens_details") or {}).get("text_tokens", tin)
    image_in = (usage.get("input_tokens_details") or {}).get("image_tokens", 0)
    text_rate, image_in_rate, image_out_rate = PRICING.get(args.model, (0.0, 0.0, 0.0))
    cost = (text_in * text_rate + image_in * image_in_rate + tout * image_out_rate) / 1e6

    # Provenance sits next to the image, named for it, so the two travel
    # together. An illustration whose prompt has been lost cannot be revised,
    # corrected, or reproduced — it can only be regenerated from scratch.
    meta = {
        "generated_by": "scripts/generate_image.py",
        "generated_at": f"{datetime.now(timezone.utc):%Y-%m-%dT%H:%M:%SZ}",
        "model": args.model,
        "size": size,
        "quality": args.quality,
        "background": args.background,
        "images": [str(p) for p in written],
        "prompt_file": args.prompt,
        "prompt": prompt,
        "revised_prompt": data[0].get("revised_prompt"),
        "usage": usage,
        "estimated_cost_usd": round(cost, 4),
    }
    meta_path = out.with_suffix(out.suffix + ".json")
    meta_path.write_text(json.dumps(meta, indent=2) + "\n")
    print(f"wrote {meta_path}")
    if tin or tout:
        print(f"  tokens in={tin:,} out={tout:,}  cost=${cost:.4f}")
    if data[0].get("revised_prompt"):
        print("  note: the model revised the prompt — see the .json for what it rendered")


if __name__ == "__main__":
    main()
