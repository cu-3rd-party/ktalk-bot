import pytest

import ktalk_bot


def test_missing_auth_file_raises_io_error():
    with pytest.raises(OSError):
        ktalk_bot.get_history(auth_file="does-not-exist.txt", max_pages=1, page_size=1)


def test_client_exposes_history_method():
    client = ktalk_bot.KTalkClient(auth_file="does-not-exist.txt")
    assert hasattr(client, "get_history")
