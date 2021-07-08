#ifndef RPC_CONNECTION_H
#define RPC_CONNECTION_H

#include <rpc/rpc.h>

#ifdef __cplusplus
extern "C" {
#endif

/// Deinit RPC-Connection
void rpc_deinitialize(CLIENT *client);

#ifdef __cplusplus
}
#endif

#endif //RPC_CONNECTION_H
