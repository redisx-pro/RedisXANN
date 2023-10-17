class DistanceMetric:
    Euclidean = "L2"  # an alias to L2
    L2 = "L2"
    InnerProduct = "IP"
    Jaccard = "JACCARD"
    Cosine = "COS"
    L2sq = "L2SQ",
    Pearson = "PEARSON",
    Haversine = "HAVERSINE",
    Hamming = "HAMMING",
    Tanimoto = "TANIMOTO",
    Sorensen = "SORENSEN",


# downcast
class UsearchQuantizationType:
    F64 = "F64"
    F32 = "F32"
    F16 = "F16"
    I8 = "I8"
    B1 = "B1"


class CmdName:
    USEARCH_CREATE_INDEX = "USEARCH.INDEX.CREATE"
    USEARCH_GET_INDEX = "USEARCH.INDEX.GET"
    USEARCH_DEL_INDEX = "USEARCH.INDEX.DEL"
    USEARCH_ADD_NODE = "USEARCH.NODE.ADD"
    USEARCH_GET_NODE = "USEARCH.NODE.GET"
    USEARCH_DEL_NODE = "USEARCH.NODE.DEL"
    USEARCH_ADD_ID_NODE = "USEARCH.NODE.ADD_ID"
    USEARCH_GET_ID_NODE = "USEARCH.NODE.GET_ID"
    USEARCH_DEL_ID_NODE = "USEARCH.NODE.DEL_ID"
    USEARCH_SEARCH_KANN = "USEARCH.SEARCH.KANN"
