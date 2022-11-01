import pytest
import pydeno


@pytest.fixture(scope="session", autouse=True)
def runtime(request):
    runtime = pydeno.DenoRuntime(
        preload_script="""
        function echo(...args) { 
            return args 
        }
    """
    )
    return runtime


def test_call_no_args(runtime):
    result = runtime.call("echo")
    assert result == []


def test_call_int_args(runtime):
    result = runtime.call("echo", 1, 2, 3)
    assert result == [1, 2, 3]


def test_call_float_args(runtime):
    result = runtime.call("echo", 1.5, 2.4)
    assert result == [1.5, 2.4]


def test_call_boolean_args(runtime):
    result = runtime.call("echo", True, False)
    assert result == [True, False]


def test_call_string_args(runtime):
    result = runtime.call("echo", "string")
    assert result == ["string"]


def test_call_array_args(runtime):
    result = runtime.call("echo", ["string", True, 1], [False])
    assert result == [["string", True, 1], [False]]


def test_call_object_args(runtime):
    result = runtime.call("echo", {"testing": True, "and": 123}, {"another": "yes"})
    assert result == [{"testing": True, "and": 123}, {"another": "yes"}]


def test_call_none_args(runtime):
    result = runtime.call("echo", None)
    assert result == [None]


def test_call_tuple_args(runtime):
    result = runtime.call("echo", ("string", True, 1), (False))
    assert result == [["string", True, 1], False]


# TODO: with timeout
