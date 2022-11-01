import pytest
import pydeno


@pytest.fixture(scope="session", autouse=True)
def runtime(request):
    return pydeno.DenoRuntime()


def test_invalid_syntax_throw_v8_exception(runtime):
    with pytest.raises(pydeno.V8Exception):
        runtime.eval("{invalid:")
