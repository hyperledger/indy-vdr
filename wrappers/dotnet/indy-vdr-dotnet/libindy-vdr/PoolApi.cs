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
        private static void GetPoolStatusCallbackMethod(long callback_id, int err, string result)
        {
            var taskCompletionSource = PendingCallbacks.Remove<string>(callback_id);
            //if (!CallbackHelper.CheckCallback(taskCompletionSource, err))
            //    return;
            if (err != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }

            taskCompletionSource.SetResult(result);
        }
        private static GetPoolStatusCompletedDelegate GetPoolStatusCallback = GetPoolStatusCallbackMethod;

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

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }

            return poolHandle;
        }

        public static async Task<int> RefreshPoolAsync(
            uint poolHandle)
        {
            int errorCode = NativeMethods.indy_vdr_pool_refresh(
                poolHandle);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }

            return errorCode;
        }

        public static async Task<string> GetPoolStatusAsync(
            uint poolHandle)
        {
            //ParamGuard needed?

            var taskCompletionSource = new TaskCompletionSource<string>();
            var callbackId = PendingCallbacks.Add(taskCompletionSource);

            int errorCode = NativeMethods.indy_vdr_pool_get_status(
                poolHandle,
                GetPoolStatusCallback,
                callbackId
                );

            /**
            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }**/

            //CallbackHelper needed?

            return await taskCompletionSource.Task;
            //return errorCode;
        }

        public static async Task<int> GetPoolTransactionsAsync(
            uint poolHandle)
        {
            int errorCode = NativeMethods.indy_vdr_pool_get_transactions(
                poolHandle);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }

            return errorCode;
        }

        public static async Task<int> GetPoolVerifiersAsync(
            uint poolHandle)
        {
            int errorCode = NativeMethods.indy_vdr_pool_get_verifiers(
                poolHandle);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }

            return errorCode;
        }

        public static async Task<int> SubmitPoolActionAsync(
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

            int errorCode = NativeMethods.indy_vdr_pool_submit_action(
                poolHandle,
                requestHandle,
                FfiStr.Create(nodesJson),
                timeout);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }

            return errorCode;
        }

        public static async Task<int> SubmitPoolRequestAsync(
            uint poolHandle,
            uint requestHandle)
        {
            int errorCode = NativeMethods.indy_vdr_pool_submit_request(
                poolHandle,
                requestHandle);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }

            return errorCode;
        }

        public static async Task<int> ClosePoolAsync(
            uint poolHandle)
        {
            int errorCode = NativeMethods.indy_vdr_pool_close(
                poolHandle);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }

            return errorCode;
        }
    }
}
