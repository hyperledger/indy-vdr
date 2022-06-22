using FluentAssertions;
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
            uint actual = await PoolApi.CreatePoolAsync(null, _genesisFilePath, null);

            //Assert
            _ = actual.Should().NotBe(0);
        }

        [Test, TestCase(TestName = "RefreshPoolAsync call returns a result string.")]
        public async Task RefreshPoolAsyncWorks()
        {
            //Arrange
            uint poolHandle = await PoolApi.CreatePoolAsync(null, _genesisFilePath, null);

            //Act
            bool actual = await PoolApi.RefreshPoolAsync(poolHandle);

            //Assert
            _ = actual.Should().Be(true);
        }

        [Test, TestCase(TestName = "GetPoolStatusAsync call returns a result string.")]
        public async Task GetPoolStatusAsyncWorks()
        {
            //Arrange
            uint poolHandle = await PoolApi.CreatePoolAsync(null, _genesisFilePath, null);

            //Act
            string actual = await PoolApi.GetPoolStatusAsync(poolHandle);

            //Assert
            _ = actual.Should().NotBe("");
        }

        [Test, TestCase(TestName = "GetPoolTransactionsAsync call returns a result string.")]
        public async Task GetPoolTransactionsAsyncWorks()
        {
            //Arrange
            uint poolHandle = await PoolApi.CreatePoolAsync(null, _genesisFilePath, null);

            //Act
            string actual = await PoolApi.GetPoolTransactionsAsync(poolHandle);

            //Assert
            _ = actual.Should().NotBe("");
        }

        [Test, TestCase(TestName = "GetPoolVerifiersAsync call returns a result string.")]
        public async Task GetPoolVerifiersAsyncWorks()
        {
            //Arrange
            uint poolHandle = await PoolApi.CreatePoolAsync(null, _genesisFilePath, null);

            //Act
            string actual = await PoolApi.GetPoolVerifiersAsync(poolHandle);

            //Assert
            _ = actual.Should().NotBe("");
        }

        [Test, TestCase(TestName = "SubmitPoolRequestAsync call returns a result string.")]
        public async Task SubmitPoolRequestAsyncWorks()
        {
            //Arrange
            uint poolHandle = await PoolApi.CreatePoolAsync(null, _genesisFilePath, null);
            uint requestHandle = await LedgerApi.BuildGetSchemaRequestAsync("9vBvpoNHmqiDu4pAUVVue7:2:Boarding Pass:1.0");

            //Act
            string actual = await PoolApi.SubmitPoolRequestAsync(poolHandle, requestHandle);

            //Assert
            _ = actual.Should().NotBe("");
        }

        [Test, TestCase(TestName = "SubmitPoolActionAsync call returns a result string.")]
        public async Task SubmitPoolActionAsyncWorks()
        {
            //Arrange
            uint poolHandle = await PoolApi.CreatePoolAsync(null, _genesisFilePath, null);
            uint requestHandle = await LedgerApi.BuildGetSchemaRequestAsync("9vBvpoNHmqiDu4pAUVVue7:2:Boarding Pass:1.0");
            List<string> nodes = new() { "xsvalidatorec2irl", "vnode1", "danube", "FoundationBuilder" };

            //Act
            string actual = await PoolApi.SubmitPoolActionAsync(poolHandle, requestHandle, nodes);

            //Assert
            _ = actual.Should().NotBe("");
        }

        [Test, TestCase(TestName = "ClosePoolAsync call returns errorcode 0.")]
        public async Task ClosePoolAsyncWorks()
        {
            //Arrange
            uint poolHandle = await PoolApi.CreatePoolAsync(null, _genesisFilePath, null);

            //Act
            int actual = await PoolApi.ClosePoolAsync(poolHandle);

            //Assert
            _ = actual.Should().Be(0);
        }
    }
}
