"""End-to-end: runs the built plugin through solidcraft-cli and checks the boxes it made."""


def run(cli, pkg, approval):
    def timber(**params):
        return {"command": "plugin.run", "params": {"path": pkg, "approval": approval, "entry": "timber", "params": params}}

    def select(name):
        return {"command": "select.set", "params": {"items": [{"type": "body", "name": name}]}}

    measure = {"command": "inspect.measure", "params": {}}
    script = [
        # a bearer, 6x2 on edge along X
        timber(size="6x2", length="3000"),
        select("6x2"),
        # five 4x2 joists across it on top, 400 centres, from its start
        timber(size="4x2", length="2000", place="square", at="top", count=5, spacing="400"),
        select("4x2 #1"),
        # a rim joist across the far end of the first joist
        timber(size="4x2", length="1647", place="square", at="end", along="start"),
        select("6x2"),
        # a second bearer parallel to the first, 1800 centres, same length
        timber(size="6x2", length="0", place="parallel", spacing="1800"),
        # a 4x4 post, and a 3x2 rail on its side flush with its top
        timber(size="4x4", length="1000", direction="z", name="Post"),
        select("Post"),
        timber(size="3x2", length="1200", place="square", at="side", along="end", name="Rail"),
        # a custom piece and a flat batten
        timber(size="custom", custom_thickness="22", custom_width="150", length="500", lie="flat", name="Board"),
        # inches: a plain number is inches, a value with its own unit is converted
        timber(size="2x2", inches=True, length="8' 6\"", count=2, spacing="16", name="Imperial"),
        timber(size="2x2", inches=True, length="1200 mm", name="Mixed"),
        measure,
        {"command": "document.inspect", "params": {"bodies": False, "timeline": False}},
    ]
    results = cli(script)
    boxes = {b["name"]: [round(v, 3) for v in b["bbox"]["min"] + b["bbox"]["max"]] for b in results[-2]["bodies"]}
    params = {p["name"]: p for p in results[-1]["params"]}
    print("Imperial_length:", params["Imperial_length"])
    assert abs(params["Imperial_length"]["value"] - 2590.8) < 1e-6 or abs(params["Imperial_length"]["value"] - 102) < 1e-6
    # the bearer lies on edge: 150 up, 47 across
    expect = {"6x2": [0, 0, 0, 3000, 47, 150]}
    for i in range(5):
        x = 400 * i
        expect[f"4x2 #{i + 1}"] = [x, -976.5, 150, x + 47, 1023.5, 250]
    # across the end of joist 1 (which runs along Y, its end at Y 1023.5): along X, centred on it
    expect["4x2 2"] = [-800, 1023.5, 150, 847, 1070.5, 250]
    expect["6x2 2"] = [0, 1800, 0, 3000, 1847, 150]
    expect["Post"] = [0, 0, 0, 100, 100, 1000]
    expect["Rail"] = [-550, 100, 925, 650, 147, 1000]
    expect["Board"] = [0, 0, 0, 500, 150, 22]
    expect["Imperial #1"] = [0, 0, 0, 2590.8, 47, 50]
    expect["Imperial #2"] = [0, 406.4, 0, 2590.8, 453.4, 50]
    expect["Mixed"] = [0, 0, 0, 1200, 47, 50]
    for name, want in expect.items():
        got = boxes.get(name)
        assert got == [float(v) for v in want], f"{name}: got {got}, want {want}"
        print("ok  ", name, got)
    assert len(boxes) == len(expect), f"unexpected bodies: {sorted(set(boxes) - set(expect))}"
    print("PASS end-to-end")
