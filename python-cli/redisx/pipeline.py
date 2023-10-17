import redis.client
import redis.cluster

from redisx.commands import RedisXCommands


# wrap redis pipeline func
class Pipeline(redis.client.Pipeline, RedisXCommands):
    def __init__(
        self,
        connection_pool,
        response_callbacks,
        transaction,
        shard_hint,
    ):
        redis.client.Pipeline.__init__(
            self,
            connection_pool,
            response_callbacks,
            transaction,
            shard_hint,
        )


# wrap redis cluster pipeline func
class ClusterPipeline(redis.cluster.ClusterPipeline, RedisXCommands):
    def __init__(
        self,
        nodes_manager,
        commands_parser,
        result_callbacks=None,
        cluster_response_callbacks=None,
        startup_nodes=None,
        read_from_replicas=False,
        cluster_error_retry_attempts=5,
        reinitialize_steps=10,
        **kwargs,
    ):
        redis.cluster.ClusterPipeline.__init__(
            self,
            nodes_manager,
            commands_parser,
            result_callbacks,
            cluster_response_callbacks,
            startup_nodes,
            read_from_replicas,
            cluster_error_retry_attempts,
            reinitialize_steps,
            **kwargs,
        )
