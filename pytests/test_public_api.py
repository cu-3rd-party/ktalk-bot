import pytest

import ktalk_bot


def test_invalid_cookie_bundle_raises_value_error():
    with pytest.raises(ValueError):
        ktalk_bot.create_engine("not-a-cookie")


def test_client_exposes_engine_methods():
    client = ktalk_bot.KTalkClient("sessionToken=test-token; ngtoken=warm")
    assert hasattr(client, "get_history")
    assert hasattr(client, "renew_cookies")
    assert hasattr(client, "join_room")
