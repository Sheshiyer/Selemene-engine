from bridges.hermes.tools import ENGINES, WORKFLOWS, SUPPORTED_WORKFLOWS, get_noesis_tools


def test_public_catalogue_has_canonical_mirrors():
    assert len(ENGINES) == 17
    assert "raaga" in ENGINES
    assert "financial-biosensor" not in ENGINES
    assert "biofield-capture" not in ENGINES
    assert len(WORKFLOWS) == 6
    assert list(SUPPORTED_WORKFLOWS) == [
        "birth-blueprint", "daily-practice", "decision-support", "self-inquiry", "creative-expression"
    ]


def test_generated_tools_mark_full_spectrum_unsupported():
    tools = get_noesis_tools(include_meta=False)
    full = next(tool for tool in tools if tool["function"]["name"] == "noesis_workflow_full_spectrum")
    assert full["function"]["x-selemene-status"] == "unsupported"
    assert "Unsupported" in full["function"]["description"]
    assert all(tool["function"]["x-selemene-runtime"] == "protected-rust" for tool in tools)


def test_subset_rejects_stale_or_unknown_identity():
    try:
        get_noesis_tools(include_engines=["tarot", "financial-biosensor"])
    except ValueError:
        pass
    else:
        raise AssertionError("stale identity was accepted")
