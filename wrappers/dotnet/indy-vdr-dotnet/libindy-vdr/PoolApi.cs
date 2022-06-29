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
        public static async Task<IntPtr> CreatePoolAsync(
            string transactions = null,
            string transactionsPath = null,
            Dictionary<string, float> nodeWeights = null)
        {
            IntPtr poolHandle = new();
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
            IntPtr poolHandle)
        {
            TaskCompletionSource<bool> taskCompletionSource = new TaskCompletionSource<bool>();
            long callbackId = PendingCallbacks.Add(taskCompletionSource);

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
        private static void PoolRefreshCallbackMethod(long callbackId, int errorCode)
        {
            TaskCompletionSource<bool> taskCompletionSource = PendingCallbacks.Remove<bool>(callbackId);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = ErrorApi.GetCurrentErrorAsync().GetAwaiter().GetResult();
                taskCompletionSource.SetException(IndyVdrException.FromSdkError(error));
                return;
            }
            taskCompletionSource.SetResult(true);
        }
        private static readonly PoolRefreshCompletedDelegate PoolRefreshCallback = PoolRefreshCallbackMethod;
        #endregion

        #region GetPoolStatusAsync
        public static async Task<string> GetPoolStatusAsync(
            IntPtr poolHandle)
        {
            TaskCompletionSource<string> taskCompletionSource = new TaskCompletionSource<string>();
            long callbackId = PendingCallbacks.Add(taskCompletionSource);

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

        private static void PoolGetStatusCallbackMethod(long callbackId, int errorCode, string result)
        {
            TaskCompletionSource<string> taskCompletionSource = PendingCallbacks.Remove<string>(callbackId);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = ErrorApi.GetCurrentErrorAsync().GetAwaiter().GetResult();
                taskCompletionSource.SetException(IndyVdrException.FromSdkError(error));
                return;
            }
            taskCompletionSource.SetResult(result);
        }
        private static readonly PoolGetStatusCompletedDelegate PoolGetStatusCallback = PoolGetStatusCallbackMethod;
        #endregion

        #region GetPoolTransactionsAsync
        public static async Task<string> GetPoolTransactionsAsync(
            IntPtr poolHandle)
        {
            TaskCompletionSource<string> taskCompletionSource = new TaskCompletionSource<string>();
            long callbackId = PendingCallbacks.Add(taskCompletionSource);

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
        private static void PoolGetTransactionsCallbackMethod(long callbackId, int errorCode, string transactions)
        {
            TaskCompletionSource<string> taskCompletionSource = PendingCallbacks.Remove<string>(callbackId);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = ErrorApi.GetCurrentErrorAsync().GetAwaiter().GetResult();
                taskCompletionSource.SetException(IndyVdrException.FromSdkError(error));
                return;
            }
            taskCompletionSource.SetResult(transactions);
        }
        private static readonly PoolGetTransactionsCompletedDelegate PoolGetTransactionsCallback = PoolGetTransactionsCallbackMethod;
        #endregion

        #region GetPoolVerifiersAsync
        public static async Task<string> GetPoolVerifiersAsync(
            IntPtr poolHandle)
        {
            TaskCompletionSource<string> taskCompletionSource = new TaskCompletionSource<string>();
            long callbackId = PendingCallbacks.Add(taskCompletionSource);

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
        private static void PoolGetVerifiersCallbackMethod(long callbackId, int errorCode, string verifiers)
        {
            TaskCompletionSource<string> taskCompletionSource = PendingCallbacks.Remove<string>(callbackId);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = ErrorApi.GetCurrentErrorAsync().GetAwaiter().GetResult();
                taskCompletionSource.SetException(IndyVdrException.FromSdkError(error));
                return;
            }
            taskCompletionSource.SetResult(verifiers);
        }
        private static readonly PoolGetVerifiersCompletedDelegate PoolGetVerifiersCallback = PoolGetVerifiersCallbackMethod;
        #endregion

        #region SubmitPoolActionAsync
        public static async Task<string> SubmitPoolActionAsync(
            IntPtr poolHandle,
            IntPtr requestHandle,
            List<string> nodeAliases = null,
            int timeout = -1)
        {
            string nodesJson = null;
            if (nodeAliases != null && nodeAliases.Any())
            {
                nodesJson = JsonConvert.SerializeObject(nodeAliases);
            }

            TaskCompletionSource<string> taskCompletionSource = new TaskCompletionSource<string>();
            long callbackId = PendingCallbacks.Add(taskCompletionSource);

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
        private static void PoolSubmitActionCallbackMethod(long callbackId, int errorCode, string result)
        {
            TaskCompletionSource<string> taskCompletionSource = PendingCallbacks.Remove<string>(callbackId);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = ErrorApi.GetCurrentErrorAsync().GetAwaiter().GetResult();
                taskCompletionSource.SetException(IndyVdrException.FromSdkError(error));
                return;
            }
            taskCompletionSource.SetResult(result);
        }
        private static readonly PoolSubmitActionCompletedDelegate PoolSubmitActionCallback = PoolSubmitActionCallbackMethod;
        #endregion

        #region SubmitPoolRequestAsync
        public static async Task<string> SubmitPoolRequestAsync(
            IntPtr poolHandle,
            IntPtr requestHandle)
        {
            TaskCompletionSource<string> taskCompletionSource = new TaskCompletionSource<string>();
            long callbackId = PendingCallbacks.Add(taskCompletionSource);

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
        private static void PoolSubmitRequestCallbackMethod(long callbackId, int errorCode, string result)
        {
            TaskCompletionSource<string> taskCompletionSource = PendingCallbacks.Remove<string>(callbackId);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = ErrorApi.GetCurrentErrorAsync().GetAwaiter().GetResult();
                taskCompletionSource.SetException(IndyVdrException.FromSdkError(error));
                return;
            }
            taskCompletionSource.SetResult(result);
        }
        private static readonly PoolSubmitRequestCompletedDelegate PoolSubmitRequestCallback = PoolSubmitRequestCallbackMethod;
        #endregion

        #region ClosePoolAsync
        public static async Task<int> ClosePoolAsync(
            IntPtr poolHandle)
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
