import pytest
import pydeno


@pytest.fixture(scope="session", autouse=True)
def runtime(request):
    return pydeno.DenoRuntime()


def test_number_conversion(runtime):
    result = runtime.eval("1.5")
    assert isinstance(result, float)
    assert result == 1.5


def test_integer_conversion(runtime):
    result = runtime.eval("1")
    assert isinstance(result, int)
    assert result == 1


def test_string_conversion(runtime):
    result = runtime.eval("'test'")
    assert isinstance(result, str)
    assert result == "test"


def test_undefined_conversion(runtime):
    result = runtime.eval("undefined")
    assert result is None


def test_null_conversion(runtime):
    result = runtime.eval("null")
    assert result is None


def test_boolean_conversion(runtime):
    result = runtime.eval("false")
    assert isinstance(result, bool)
    assert result == False

    result = runtime.eval("true")
    assert isinstance(result, bool)
    assert result == True


def test_list_conversion(runtime):
    result = runtime.eval("[1,2,3]")
    assert isinstance(result, list)
    assert result == [1, 2, 3]


def test_nested_list_conversion(runtime):
    result = runtime.eval("[1, [2,3], {object: true}]")
    assert isinstance(result, list)
    assert result == [1, [2, 3], {"object": True}]


def test_object_conversion(runtime):
    result = runtime.eval("({test: true})")
    assert isinstance(result, dict)
    assert result == {"test": True}


def test_nested_object_conversion(runtime):
    result = runtime.eval("({test: {'nested': [1,2,3]}})")
    assert isinstance(result, dict)
    assert result == {"test": {"nested": [1, 2, 3]}}


def test_object_with_numeric_keys_conversion(runtime):
    result = runtime.eval("({1: false, test: true})")
    assert isinstance(result, dict)
    assert result == {1: False, "test": True}


# TODO: sets
# TODO: classes
