using indy_vdr_dotnet.utils;
using Newtonsoft.Json;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using static indy_vdr_dotnet.libindy_vdr.NativeMethods;
using static indy_vdr_dotnet.models.Structures;

namespace indy_vdr_dotnet.libindy_vdr
{
    public class PoolApi
    {
        #region CreatePoolAsync
        public static async Task<uint> CreatePoolAsync(
            string transactions = null,
            string transactionsPath = null,
            Dictionary<string, float> nodeWeights = null)
        {
            uint poolHandle = 0;
            string paramsJson = JsonConvert.SerializeObject(new 
            {
                transactions,
                transactions_path = transactionsPath,
                node_weights = nodeWeights
            });

            int errorCode = NativeMethods.indy_vdr_pool_create(
                FfiStr.Create(paramsJson),
                ref poolHandle);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }

            return poolHandle;
        }
        #endregion

        #region RefreshPoolAsync
        public static async Task<bool> RefreshPoolAsync(
            uint poolHandle)
        {
            var taskCompletionSource = new TaskCompletionSource<bool>();
            var callbackId = PendingCallbacks.Add(taskCompletionSource);

            int errorCode = NativeMethods.indy_vdr_pool_refresh(
                poolHandle,
                PoolRefreshCallback,
                callbackId
                );

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }

            return await taskCompletionSource.Task;
        }
        private static void PoolRefreshCallbackMethod(long callback_id, int err)
        {
            var taskCompletionSource = PendingCallbacks.Remove<bool>(callback_id);

            if (err != (int)ErrorCode.Success)
            {
                string error = ErrorApi.GetCurrentErrorAsync().GetAwaiter().GetResult();
                taskCompletionSource.SetException(IndyVdrException.FromSdkError(error));
                return;
            }
            taskCompletionSource.SetResult(true);
        }
        private static PoolRefreshCompletedDelegate PoolRefreshCallback = PoolRefreshCallbackMethod;
        #endregion

        #region GetPoolStatusAsync
        public static async Task<string> GetPoolStatusAsync(
            uint poolHandle)
        {
            var taskCompletionSource = new TaskCompletionSource<string>();
            var callbackId = PendingCallbacks.Add(taskCompletionSource);

            int errorCode = NativeMethods.indy_vdr_pool_get_status(
                poolHandle,
                PoolGetStatusCallback,
                callbackId
                );

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }

            return await taskCompletionSource.Task;
        }

        private static void PoolGetStatusCallbackMethod(long callback_id, int err, string result)
        {
            var taskCompletionSource = PendingCallbacks.Remove<string>(callback_id);

            if (err != (int)ErrorCode.Success)
            {
                string error = ErrorApi.GetCurrentErrorAsync().GetAwaiter().GetResult();
                taskCompletionSource.SetException(IndyVdrException.FromSdkError(error));
                return;
            }
            taskCompletionSource.SetResult(result);
        }
        private static PoolGetStatusCompletedDelegate PoolGetStatusCallback = PoolGetStatusCallbackMethod;
        #endregion

        #region GetPoolTransactionsAsync
        public static async Task<string> GetPoolTransactionsAsync(
            uint poolHandle)
        {
            var taskCompletionSource = new TaskCompletionSource<string>();
            var callbackId = PendingCallbacks.Add(taskCompletionSource);

            int errorCode = NativeMethods.indy_vdr_pool_get_transactions(
                poolHandle,
                PoolGetTransactionsCallback,
                callbackId);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }

            return await taskCompletionSource.Task;
        }
        private static void PoolGetTransactionsCallbackMethod(long callback_id, int err, string result)
        {
            var taskCompletionSource = PendingCallbacks.Remove<string>(callback_id);

            if (err != (int)ErrorCode.Success)
            {
                string error = ErrorApi.GetCurrentErrorAsync().GetAwaiter().GetResult();
                taskCompletionSource.SetException(IndyVdrException.FromSdkError(error));
                return;
            }
            taskCompletionSource.SetResult(result);
        }
        private static PoolGetTransactionsCompletedDelegate PoolGetTransactionsCallback = PoolGetTransactionsCallbackMethod;
        #endregion

        #region GetPoolVerifiersAsync
        public static async Task<string> GetPoolVerifiersAsync(
            uint poolHandle)
        {
            var taskCompletionSource = new TaskCompletionSource<string>();
            var callbackId = PendingCallbacks.Add(taskCompletionSource);

            int errorCode = NativeMethods.indy_vdr_pool_get_verifiers(
                poolHandle,
                PoolGetVerifiersCallback,
                callbackId);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }

            return await taskCompletionSource.Task;
        }
        private static void PoolGetVerifiersCallbackMethod(long callback_id, int err, string result)
        {
            var taskCompletionSource = PendingCallbacks.Remove<string>(callback_id);

            if (err != (int)ErrorCode.Success)
            {
                string error = ErrorApi.GetCurrentErrorAsync().GetAwaiter().GetResult();
                taskCompletionSource.SetException(IndyVdrException.FromSdkError(error));
                return;
            }
            taskCompletionSource.SetResult(result);
        }
        private static PoolGetVerifiersCompletedDelegate PoolGetVerifiersCallback = PoolGetVerifiersCallbackMethod;
        #endregion

        #region SubmitPoolActionAsync
        public static async Task<string> SubmitPoolActionAsync(
            uint poolHandle,
            uint requestHandle,
            List<string> nodeAliases = null,
            int timeout = -1)
        {
            string nodesJson = null;
            if (nodeAliases != null && nodeAliases.Any())
            {
                nodesJson = JsonConvert.SerializeObject(nodeAliases);
            }

            var taskCompletionSource = new TaskCompletionSource<string>();
            var callbackId = PendingCallbacks.Add(taskCompletionSource);

            int errorCode = NativeMethods.indy_vdr_pool_submit_action(
                poolHandle,
                requestHandle,
                FfiStr.Create(nodesJson),
                timeout,
                PoolSubmitActionCallback,
                callbackId);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }

            return await taskCompletionSource.Task;
        }
        private static void PoolSubmitActionCallbackMethod(long callback_id, int err, string result)
        {
            var taskCompletionSource = PendingCallbacks.Remove<string>(callback_id);

            if (err != (int)ErrorCode.Success)
            {
                string error = ErrorApi.GetCurrentErrorAsync().GetAwaiter().GetResult();
                taskCompletionSource.SetException(IndyVdrException.FromSdkError(error));
                return;
            }
            taskCompletionSource.SetResult(result);
        }
        private static PoolSubmitActionCompletedDelegate PoolSubmitActionCallback = PoolSubmitActionCallbackMethod;
        #endregion

        #region SubmitPoolRequestAsync
        public static async Task<string> SubmitPoolRequestAsync(
            uint poolHandle,
            uint requestHandle)
        {
            var taskCompletionSource = new TaskCompletionSource<string>();
            var callbackId = PendingCallbacks.Add(taskCompletionSource);

            int errorCode = NativeMethods.indy_vdr_pool_submit_request(
                poolHandle,
                requestHandle,
                PoolSubmitRequestCallback,
                callbackId);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }

            return await taskCompletionSource.Task;
        }
        private static void PoolSubmitRequestCallbackMethod(long callback_id, int err, string result)
        {
            var taskCompletionSource = PendingCallbacks.Remove<string>(callback_id);

            if (err != (int)ErrorCode.Success)
            {
                string error = ErrorApi.GetCurrentErrorAsync().GetAwaiter().GetResult();
                taskCompletionSource.SetException(IndyVdrException.FromSdkError(error));
                return;
            }
            taskCompletionSource.SetResult(result);
        }
        private static PoolSubmitRequestCompletedDelegate PoolSubmitRequestCallback = PoolSubmitRequestCallbackMethod;
        #endregion

        #region ClosePoolAsync
        public static async Task<int> ClosePoolAsync(
            uint poolHandle)
        {
            int errorCode = NativeMethods.indy_vdr_pool_close(
                poolHandle);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }

            return errorCode;
        }
        #endregion
    }
}
