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
            string genesisFile = Path.Combine(currentDirectory, @"..\..\..\Resources\ew_builder");
            _genesisFilePath = Path.GetFullPath(genesisFile);
        }

        [Test, TestCase(TestName = "CreatePoolAsync call returns request handle.")]
        public async Task CreatePoolAsyncWorks()
        {
            //Arrange

            //Act
            IntPtr actual = await PoolApi.CreatePoolAsync(null, _genesisFilePath, null);

            //Assert
            _ = actual.Should().NotBe(new IntPtr());
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

        [Test, TestCase(TestName = "RefreshPoolAsync call returns a result bool.")]
        public async Task RefreshPoolAsyncWorks()
        {
            //Arrange
            IntPtr poolHandle = await PoolApi.CreatePoolAsync(null, _genesisFilePath, null);

            //Act
            bool actual = await PoolApi.RefreshPoolAsync(poolHandle);

            //Assert
            _ = actual.Should().Be(true);
        }

        [Test, TestCase(TestName = "RefreshPoolAsync call throws.")]
        public async Task RefreshPoolAsyncThrows()
        {
            //Arrange
            IntPtr poolHandle = new();

            //Act
            Func<Task> func = async () => await PoolApi.RefreshPoolAsync(poolHandle);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }

        [Test, TestCase(TestName = "GetPoolStatusAsync call returns a result string.")]
        public async Task GetPoolStatusAsyncWorks()
        {
            //Arrange
            IntPtr poolHandle = await PoolApi.CreatePoolAsync(null, _genesisFilePath, null);

            //Act
            string actual = await PoolApi.GetPoolStatusAsync(poolHandle);

            //Assert
            _ = actual.Should().NotBe("");
        }

        [Test, TestCase(TestName = "GetPoolStatusAsync call throws.")]
        public async Task GetPoolStatusAsyncThrows()
        {
            //Arrange
            IntPtr poolHandle = new();

            //Act
            Func<Task> func = async () =>  await PoolApi.GetPoolStatusAsync(poolHandle);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }

        [Test, TestCase(TestName = "GetPoolTransactionsAsync call returns a result string.")]
        public async Task GetPoolTransactionsAsyncWorks()
        {
            //Arrange
            IntPtr poolHandle = await PoolApi.CreatePoolAsync(null, _genesisFilePath, null);

            //Act
            string actual = await PoolApi.GetPoolTransactionsAsync(poolHandle);

            //Assert
            _ = actual.Should().NotBe("");
        }

        [Test, TestCase(TestName = "GetPoolTransactionsAsync call throws.")]
        public async Task GetPoolTransactionsAsyncThrows()
        {
            //Arrange
            IntPtr poolHandle = new();

            //Act
            Func<Task> func = async () => await PoolApi.GetPoolTransactionsAsync(poolHandle);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }

        [Test, TestCase(TestName = "GetPoolVerifiersAsync call returns a result string.")]
        public async Task GetPoolVerifiersAsyncWorks()
        {
            //Arrange
            IntPtr poolHandle = await PoolApi.CreatePoolAsync(null, _genesisFilePath, null);

            //Act
            string actual = await PoolApi.GetPoolVerifiersAsync(poolHandle);

            //Assert
            _ = actual.Should().NotBe("");
        }

        [Test, TestCase(TestName = "GetPoolVerifiersAsync call throws.")]
        public async Task GetPoolVerifiersAsyncThrows()
        {
            //Arrange
            IntPtr poolHandle = new();

            //Act
            Func<Task> func = async () => await PoolApi.GetPoolVerifiersAsync(poolHandle);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }

        [Test, TestCase(TestName = "SubmitPoolRequestAsync call returns a result string.")]
        public async Task SubmitPoolRequestAsyncWorks()
        {
            //Arrange
            IntPtr poolHandle = await PoolApi.CreatePoolAsync(null, _genesisFilePath, null);
            IntPtr requestHandle = await LedgerApi.BuildGetSchemaRequestAsync("9vBvpoNHmqiDu4pAUVVue7:2:Boarding Pass:1.0");

            //Act
            string actual = await PoolApi.SubmitPoolRequestAsync(poolHandle, requestHandle);

            //Assert
            _ = actual.Should().NotBe("");
        }

        [Test, TestCase(TestName = "SubmitPoolRequestAsync call throws.")]
        public async Task SubmitPoolRequestAsyncThrows()
        {
            //Arrange
            IntPtr poolHandle = new();
            IntPtr requestHandle = await LedgerApi.BuildGetSchemaRequestAsync("9vBvpoNHmqiDu4pAUVVue7:2:Boarding Pass:1.0");

            //Act
            Func<Task> func = async () => await PoolApi.SubmitPoolRequestAsync(poolHandle, requestHandle);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }

        [Test, TestCase(TestName = "SubmitPoolActionAsync call returns a result string.")]
        public async Task SubmitPoolActionAsyncWorks()
        {
            //Arrange
            IntPtr poolHandle = await PoolApi.CreatePoolAsync(null, _genesisFilePath, null);
            IntPtr requestHandle = await LedgerApi.BuildGetSchemaRequestAsync("9vBvpoNHmqiDu4pAUVVue7:2:Boarding Pass:1.0");
            List<string> nodes = new() { "xsvalidatorec2irl", "vnode1", "danube", "FoundationBuilder" };

            //Act
            string actual = await PoolApi.SubmitPoolActionAsync(poolHandle, requestHandle, nodes);

            //Assert
            _ = actual.Should().NotBe("");
        }

        [Test, TestCase(TestName = "SubmitPoolActionAsync call throws.")]
        public async Task SubmitPoolActionAsyncThrows()
        {
            //Arrange
            IntPtr poolHandle = new();
            IntPtr requestHandle = await LedgerApi.BuildGetSchemaRequestAsync("9vBvpoNHmqiDu4pAUVVue7:2:Boarding Pass:1.0");
            List<string> nodes = new() { "xsvalidatorec2irl", "vnode1", "danube", "FoundationBuilder" };

            //Act
            Func<Task> func = async () => await PoolApi.SubmitPoolActionAsync(poolHandle, requestHandle, nodes);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }

        [Test, TestCase(TestName = "ClosePoolAsync call returns errorcode 0.")]
        public async Task ClosePoolAsyncWorks()
        {
            //Arrange
            IntPtr poolHandle = await PoolApi.CreatePoolAsync(null, _genesisFilePath, null);

            //Act
            int actual = await PoolApi.ClosePoolAsync(poolHandle);

            //Assert
            _ = actual.Should().Be(0);
        }

        [Test, TestCase(TestName = "ClosePoolAsync call throws.")]
        public async Task ClosePoolAsyncThrows()
        {
            //Arrange
            IntPtr poolHandle = new();

            //Act
            Func<Task> func = async () => await PoolApi.ClosePoolAsync(poolHandle);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
    }
}
