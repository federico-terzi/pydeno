import pytest
import pydeno


@pytest.fixture(scope="session", autouse=True)
def runtime(request):
    return pydeno.DenoRuntime()


def test_invalid_syntax_throw_v8_exception(runtime):
    with pytest.raises(pydeno.V8Exception):
        runtime.eval("{invalid:")


def test_eval_timeout(runtime):
    assert runtime.eval("var i = 0; while (i < 1000) {i++}; i", timeout_ms=100) == 1000

    with pytest.raises(pydeno.TimeoutException):
        runtime.eval("var i = 0; while (i < 100000000000) {i++}; i", timeout_ms=1000)

    # Make sure the runtime still works afterwards
    assert runtime.eval("1+2") == 3
