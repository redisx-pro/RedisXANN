import redis
import define
from typing import Dict, Union

from redisx import ann_usearch


# just a commands class set for extend
class RedisXCommands(
    ann_usearch.UsearchVectorCommands,
):
    pass


def parse_usearch_get_index_result(resp) -> Union[Dict, None]:
    if len(resp) == 0:
        return None
    return redis.client.pairs_to_dict(resp, decode_keys=True, decode_string_values=True)


REDISX_RESPONSE_CALLBACKS = {
    # RedisXANN Usearch Vector
    define.CmdName.USEARCH_CREATE_INDEX: redis.client.bool_ok,
    define.CmdName.USEARCH_GET_INDEX: parse_usearch_get_index_result,
}


def set_response_callback(redis: Union[redis.Redis, redis.AsyncRedis]):
    for cmd, cb in REDISX_RESPONSE_CALLBACKS.items():
        redis.set_response_callback(cmd, cb)
