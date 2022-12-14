def task_maturin_develop():
    return {
        "actions": ["maturin develop"],
        "io": {
            "capture": False,
        },
    }


def task_test():
    return {
        "actions": ["pytest -s"],
        "task_dep": ["maturin_develop"],
        "io": {
            "capture": False,
        },
    }
