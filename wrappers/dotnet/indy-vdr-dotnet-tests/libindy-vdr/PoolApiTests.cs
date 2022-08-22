using FluentAssertions;
using indy_vdr_dotnet;
using indy_vdr_dotnet.libindy_vdr;
using NUnit.Framework;
using System;
using System.Collections.Generic;
using System.IO;
using System.Threading.Tasks;

namespace indy_vdr_dotnet_tests.libindy_vdr
{
    public class PoolApiTests
    {
        private string _genesisFilePath;

        [OneTimeSetUp]
        public void OneTimeSetUp()
        {
            string currentDirectory = AppDomain.CurrentDomain.BaseDirectory;
            string genesisFile = Path.Combine(currentDirectory, @"..\..\..\Resources\genesis_builder");
            _genesisFilePath = Path.GetFullPath(genesisFile);
        }

        #region Tests for CreatePoolAsync
        [Test, TestCase(TestName = "CreatePoolAsync call returns request handle.")]
        public async Task CreatePoolAsyncWorks()
        {
            //Arrange

            //Act
            IntPtr poolHandle = await PoolApi.CreatePoolAsync(null, _genesisFilePath, null);

            //Assert
            _ = poolHandle.Should().NotBe(new IntPtr());
        }

        [Test, TestCase(TestName = "CreatePoolAsync call throws.")]
        public async Task CreatePoolAsyncThrows()
        {
            //Arrange

            //Act
            Func<Task> func = async () => await PoolApi.CreatePoolAsync(null, "", null);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion

        #region Tests for RefreshPoolAsync
        [Test, TestCase(TestName = "RefreshPoolAsync call returns a result bool.")]
        public async Task RefreshPoolAsyncWorks()
        {
            //Arrange
            IntPtr testPoolHandle = await PoolApi.CreatePoolAsync(null, _genesisFilePath, null);

            //Act
            bool actual = await PoolApi.RefreshPoolAsync(testPoolHandle);

            //Assert
            _ = actual.Should().Be(true);
        }

        [Test, TestCase(TestName = "RefreshPoolAsync call throws.")]
        public async Task RefreshPoolAsyncThrows()
        {
            //Arrange
            IntPtr testPoolHandle = new();

            //Act
            Func<Task> func = async () => await PoolApi.RefreshPoolAsync(testPoolHandle);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion

        #region Tests for GetPoolStatusAsync
        [Test, TestCase(TestName = "GetPoolStatusAsync call returns a result string.")]
        public async Task GetPoolStatusAsyncWorks()
        {
            //Arrange
            IntPtr testPoolHandle = await PoolApi.CreatePoolAsync(null, _genesisFilePath, null);

            //Act
            string actual = await PoolApi.GetPoolStatusAsync(testPoolHandle);

            //Assert
            _ = actual.Should().NotBe("");
        }

        [Test, TestCase(TestName = "GetPoolStatusAsync call throws.")]
        public async Task GetPoolStatusAsyncThrows()
        {
            //Arrange
            IntPtr testPoolHandle = new();

            //Act
            Func<Task> func = async () => await PoolApi.GetPoolStatusAsync(testPoolHandle);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion

        #region Tests for GetPoolTransactionAsync
        [Test, TestCase(TestName = "GetPoolTransactionsAsync call returns a result string.")]
        public async Task GetPoolTransactionsAsyncWorks()
        {
            //Arrange
            IntPtr testPoolHandle = await PoolApi.CreatePoolAsync(null, _genesisFilePath, null);

            //Act
            string actual = await PoolApi.GetPoolTransactionsAsync(testPoolHandle);

            //Assert
            _ = actual.Should().NotBe("");
        }

        [Test, TestCase(TestName = "GetPoolTransactionsAsync call throws.")]
        public async Task GetPoolTransactionsAsyncThrows()
        {
            //Arrange
            IntPtr testPoolHandle = new();

            //Act
            Func<Task> func = async () => await PoolApi.GetPoolTransactionsAsync(testPoolHandle);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion

        #region Tests for GetPoolVerifiersAsync
        [Test, TestCase(TestName = "GetPoolVerifiersAsync call returns a result string.")]
        public async Task GetPoolVerifiersAsyncWorks()
        {
            //Arrange
            IntPtr testPoolHandle = await PoolApi.CreatePoolAsync(null, _genesisFilePath, null);

            //Act
            string actual = await PoolApi.GetPoolVerifiersAsync(testPoolHandle);

            //Assert
            _ = actual.Should().NotBe("");
        }

        [Test, TestCase(TestName = "GetPoolVerifiersAsync call throws.")]
        public async Task GetPoolVerifiersAsyncThrows()
        {
            //Arrange
            IntPtr testPoolHandle = new();

            //Act
            Func<Task> func = async () => await PoolApi.GetPoolVerifiersAsync(testPoolHandle);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion

        #region Tests for SubmitPoolRequestAsync
        [Test, TestCase(TestName = "SubmitPoolRequestAsync call returns a result string for GET_SCHEMA request.")]
        public async Task SubmitPoolRequestAsyncWorks()
        {
            //Arrange
            IntPtr testPoolHandle = await PoolApi.CreatePoolAsync(null, _genesisFilePath, null);
            string testSchemaId = "9vBvpoNHmqiDu4pAUVVue7:2:Boarding Pass:1.0";

            IntPtr testRequestHandle = await LedgerApi.BuildGetSchemaRequestAsync(testSchemaId);
            string debug = await RequestApi.RequestGetBodyAsync(testRequestHandle);

            //Act
            string actual = await PoolApi.SubmitPoolRequestAsync(testPoolHandle, testRequestHandle);
            
            //Assert
            _ = actual.Should().NotBe("");
        }

        [Test, TestCase(TestName = "SubmitPoolRequestAsync call returns a result string for GET_CRED_DEF request.")]
        public async Task SubmitPoolRequestAsyncWorksCredDef()
        {
            //Arrange
            IntPtr testPoolHandle = await PoolApi.CreatePoolAsync(null, _genesisFilePath, null);
            string testCredDefDid = "A9Rsuu7FNquw8Ne2Smu5Nr:3:CL:15:tag";
            string testSubmitterDid = "LibindyDid111111111111";

            IntPtr testRequestHandle = await LedgerApi.BuildGetCredDefRequest(testCredDefDid, testSubmitterDid);
            string debug = await RequestApi.RequestGetBodyAsync(testRequestHandle);

            //Act
            string actual = await PoolApi.SubmitPoolRequestAsync(testPoolHandle, testRequestHandle);

            //Assert
            _ = actual.Should().NotBe("");
        }

        [Test, TestCase(TestName = "SubmitPoolRequestAsync call returns a result string for GET_ATTRIB request.")]
        public async Task SubmitPoolRequestAsyncWorksGetAttr()
        {
            //Arrange
            IntPtr testPoolHandle = await PoolApi.CreatePoolAsync(null, _genesisFilePath, null);
            string testSubmitterDid = "LibindyDid111111111111";
            string testTargetDid = "LibindyDid111111111111";

            IntPtr testRequestHandle = await LedgerApi.BuildGetAttributeRequest(testTargetDid, testSubmitterDid, "name", null, null);
            string debug = await RequestApi.RequestGetBodyAsync(testRequestHandle);
            //Act
            string actual = await PoolApi.SubmitPoolRequestAsync(testPoolHandle, testRequestHandle);

            //Assert
            _ = actual.Should().NotBe("");
        }

        [Test, TestCase(TestName = "SubmitPoolRequestAsync call throws.")]
        public async Task SubmitPoolRequestAsyncThrows()
        {
            //Arrange
            IntPtr testPoolHandle = new();
            IntPtr testRequestHandle = await LedgerApi.BuildGetSchemaRequestAsync("9vBvpoNHmqiDu4pAUVVue7:2:Boarding Pass:1.0");

            //Act
            Func<Task> func = async () => await PoolApi.SubmitPoolRequestAsync(testPoolHandle, testRequestHandle);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion

        #region Tests for SubmitPoolActionAsync
        [Test, TestCase(TestName = "SubmitPoolActionAsync call returns a result string.")]
        public async Task SubmitPoolActionAsyncWorks()
        {
            //Arrange
            IntPtr testPoolHandle = await PoolApi.CreatePoolAsync(null, _genesisFilePath, null);
            IntPtr testRequestHandle = await LedgerApi.BuildGetSchemaRequestAsync("9vBvpoNHmqiDu4pAUVVue7:2:Boarding Pass:1.0");
            List<string> testNodes = new() { "xsvalidatorec2irl", "vnode1", "danube", "FoundationBuilder" };

            //Act
            string actual = await PoolApi.SubmitPoolActionAsync(testPoolHandle, testRequestHandle, testNodes);

            //Assert
            _ = actual.Should().NotBe("");
        }

        [Test, TestCase(TestName = "SubmitPoolActionAsync call throws.")]
        public async Task SubmitPoolActionAsyncThrows()
        {
            //Arrange
            IntPtr testPoolHandle = new();
            IntPtr testRequestHandle = await LedgerApi.BuildGetSchemaRequestAsync("9vBvpoNHmqiDu4pAUVVue7:2:Boarding Pass:1.0");
            List<string> testNodes = new() { "xsvalidatorec2irl", "vnode1", "danube", "FoundationBuilder" };

            //Act
            Func<Task> func = async () => await PoolApi.SubmitPoolActionAsync(testPoolHandle, testRequestHandle, testNodes);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion

        #region Tests for ClosePoolAsync
        [Test, TestCase(TestName = "ClosePoolAsync call returns errorcode 0.")]
        public async Task ClosePoolAsyncWorks()
        {
            //Arrange
            IntPtr testPoolHandle = await PoolApi.CreatePoolAsync(null, _genesisFilePath, null);

            //Act
            int actual = await PoolApi.ClosePoolAsync(testPoolHandle);

            //Assert
            _ = actual.Should().Be(0);
        }

        [Test, TestCase(TestName = "ClosePoolAsync call throws.")]
        public async Task ClosePoolAsyncThrows()
        {
            //Arrange
            IntPtr testPoolHandle = new();

            //Act
            Func<Task> func = async () => await PoolApi.ClosePoolAsync(testPoolHandle);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion
    }
}
