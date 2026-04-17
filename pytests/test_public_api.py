import pytest

import ktalk_bot


def test_invalid_cookie_bundle_raises_value_error():
    with pytest.raises(ValueError):
        ktalk_bot.create_engine("not-a-cookie")


def test_client_exposes_engine_methods():
    client = ktalk_bot.KTalkClient("ngtoken=warm; kontur_ngtoken=hot")
    assert hasattr(client, "get_history")
    assert hasattr(client, "renew_cookies")
    assert hasattr(client, "join_room")


def test_client_can_bind_room_once():
    client = ktalk_bot.KTalkClient("ngtoken=warm; kontur_ngtoken=hot")
    client.bind_room("https://centraluniversity.ktalk.ru/demo-room")
    assert client.current_room() == "https://centraluniversity.ktalk.ru/demo-room"
