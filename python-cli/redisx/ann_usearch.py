import define
from typing import (Dict, Iterable, List, Optional, Sequence, Tuple, Union)
from functools import partial, reduce
from redis.typing import (CommandsProtocol)

VectorType = Sequence[Union[int, float]]


class TextVectorEncoder:
    SEP = bytes(",", "ascii")
    BITS = ("0", "1")

    @classmethod
    def encode(cls, vector: VectorType, is_binary=False) -> bytes:
        s = ""
        if is_binary:
            s = "[" + ",".join([cls.BITS[x] for x in vector]) + "]"
        else:
            s = "[" + ",".join(["%f" % x for x in vector]) + "]"
        return bytes(s, encoding="ascii")  # ascii is enough

    @classmethod
    def decode(cls, buf: bytes) -> Tuple[float]:
        if buf[0] != ord("[") or buf[-1] != ord("]"):
            raise ValueError("invalid text vector value")
        is_int = True
        components = buf[1:-1].split(cls.SEP)
        for x in components:
            if not x.isdigit():
                is_int = False

        if is_int:
            return tuple(int(x) for x in components)
        return tuple(float(x) for x in components)


class UsearchVectorCommands(CommandsProtocol):

    def create_index(
        self,
        name: str,
        dim: int,
        m: int = 10,
        efcon: int = 128,
        metric: str = define.DistanceMetric.IP,
        quantization: str = define.UsearchQuantizationType.F32,
        **kwargs
    ):
        """
        create a index
        cmd eg: usearch.index.create idx0 dim 3 m 10 efcon 12 metric ip quantization f32
        """
        params = reduce(lambda x, y: x + y, kwargs.items(), ())
        return self.execute_command(
            define.CmdName.USEARCH_CREATE_INDEX,
            name,
            "dim",
            dim,
            "m",
            m,
            "efcon",
            efcon,
            "metric",
            metric,
            "quantization",
            quantization,
            *params
        )

    def get_index(self, name: str):
        """
        get the infomation of an index
        """
        return self.execute_command(
            define.CmdName.USEARCH_GET_INDEX,
            name
        )

    def del_index(self, name: str):
        """
        delete an index and all its data
        """
        return self.execute_command(
            define.CmdName.USEARCH_DEL_INDEX,
            name
        )
