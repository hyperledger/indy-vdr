using FluentAssertions;
using indy_vdr_dotnet;
using indy_vdr_dotnet.libindy_vdr;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using NUnit.Framework;
using System;
using System.Collections.Generic;
using System.IO;
using System.Threading.Tasks;

namespace indy_vdr_dotnet_tests.libindy_vdr
{
    public class LedgerApiTests
    {
        private string _genesisFilePath;

        [OneTimeSetUp]
        public void OneTimeSetUp()
        {
            string currentDirectory = AppDomain.CurrentDomain.BaseDirectory;
            string genesisFile = Path.Combine(currentDirectory, @"..\..\..\Resources\genesis_builder");
            _genesisFilePath = Path.GetFullPath(genesisFile);
        }

        #region Tests for BuildAcceptanceMechanismsRequestAsync
        [Test, TestCase(TestName = "BuildAcceptanceMechanismsRequestAsync() call returns request handle.")]
        public async Task BuildAcceptanceMechanismsRequestAsyncWorks()
        {
            //Arrange 
            string testSubmitterDid = "LibindyDid111111111111";
            Dictionary<string, Dictionary<string, string>> testDict = new() { { "test", new Dictionary<string, string>() { { "description", "" } } } };
            string testAml = JsonConvert.SerializeObject(testDict);
            string testVersion = "1";
            string testAml_context = "test_aml_context";

            //Act
            IntPtr testObject = await LedgerApi.BuildAcceptanceMechanismsRequestAsync(
                testSubmitterDid,
                testAml,
                testVersion,
                testAml_context);

            //Assert
            _ = testObject.Should().NotBe(new IntPtr());
        }
        [Test, TestCase(TestName = "BuildAcceptanceMechanismsRequestAsync() call with invalid submitterDid length throws.")]
        public async Task BuildAcceptanceMechanismsRequestAsyncThrows()
        {
            //Arrange 
            string testSubmitterDid = "InvalidLength";
            Dictionary<string, Dictionary<string, string>> testDict = new() { { "test", new Dictionary<string, string>() { { "description", "" } } } };
            string testAml = JsonConvert.SerializeObject(testDict);
            string testVersion = "1";
            string testAmlContext = "test_aml_context";

            //Act
            Func<Task> func = async () => await LedgerApi.BuildAcceptanceMechanismsRequestAsync(
                testSubmitterDid,
                testAml,
                testVersion,
                testAmlContext);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }

        [Test, TestCase(TestName = "BuildGetAcceptanceMechanismsRequestAsync() call returns request handle.")]
        public async Task BuildGetAcceptanceMechanismsRequestAsyncWorks()
        {
            //Arrange 
            long testTimestamp = DateTimeOffset.Now.ToUnixTimeSeconds();

            //Act
            IntPtr testObject = await LedgerApi.BuildGetAcceptanceMechanismsRequestAsync(
                testTimestamp);

            //Assert
            _ = testObject.Should().NotBe(new IntPtr());
        }

        [Test, TestCase(TestName = "BuildGetAcceptanceMechanismsRequestAsync() call with invalid submitterDid length throws.")]
        public async Task BuildGetAcceptanceMechanismsRequestAsyncThrows()
        {
            //Arrange 
            long testTimestamp = DateTimeOffset.Now.ToUnixTimeSeconds();

            //Act
            Func<Task> func = async () => await LedgerApi.BuildGetAcceptanceMechanismsRequestAsync(
                testTimestamp,
                submitterDid: "InvalidLength");

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion

        #region Tests for BuildAttributeRequest
        [Test, TestCase(TestName = "BuildAttributeRequest() call returns request handle.")]
        public async Task BuildAttributeRequestWorks()
        {
            //Arrange 
            string testSubmitterDid = "LibindyDid111111111111";
            string testTargetDid = "LibindyDid111111111111";
            string testRaw = "{\"name\":\"Alex\"}";
            //Act
            IntPtr testObject = await LedgerApi.BuildAttributeRequest(
                testTargetDid,
                testSubmitterDid,
                null,
                testRaw,
                null);

            //Assert
            _ = testObject.Should().NotBe(new IntPtr());
        }

        [Test, TestCase(TestName = "BuildAttributeRequest() call with invalid submitterDid length throws.")]
        public async Task BuildAttributeRequestThrows()
        {
            //Arrange 
            string testSubmitterDid = "InvalidLength";
            string testTargetDid = "LibindyDid111111111111";

            //Act
            Func<Task> func = async () => await LedgerApi.BuildAttributeRequest(
                testTargetDid,
                testSubmitterDid);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }

        [Test, TestCase(TestName = "BuildGetAttributeRequest() call returns request handle.")]
        public async Task BuildGetAttributeRequestWorks()
        {
            //Arrange 
            string testSubmitterDid = "LibindyDid111111111111";
            string testTargetDid = "LibindyDid111111111111";

            //Act
            IntPtr testObject = await LedgerApi.BuildGetAttributeRequest(
                testTargetDid,
                testSubmitterDid);

            //Assert
            _ = testObject.Should().NotBe(new IntPtr());
        }

        [Test, TestCase(TestName = "BuildGetAttributeRequest() call with invalid submitterDid length throws.")]
        public async Task BuildGetAttributeRequestThrows()
        {
            //Arrange 
            string testSubmitterDid = "InvalidLength";
            string testTargetDid = "LibindyDid111111111111";

            //Act
            Func<Task> func = async () => await LedgerApi.BuildGetAttributeRequest(
                testTargetDid,
                testSubmitterDid);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion

        #region Tests for BuildCredDefRequest
        [Test, TestCase(TestName = "BuildCredDefRequest() call returns request handle.")]
        public async Task BuildCredDefRequestWorks()
        {
            //Arrange 
            string testSubmitterDid = "LibindyDid111111111111";
            string testCredDef = "{\"id\":\"testCredDefId\",\"schemaId\":\"testSchemaId\",\"type\":\"CL\",\"tag\":\"\",\"value\":{\"primary\":\"testPrimaryCredentialPublicKey\",\"revoation\":\"\"},\"ver\":\"1.0\"}";

            //Act
            IntPtr testObject = await LedgerApi.BuildCredDefRequest(
                testSubmitterDid,
                testCredDef);

            //Assert
            _ = testObject.Should().NotBe(new IntPtr());
        }

        [Test, TestCase(TestName = "BuildCredDefRequest() call with invalid submitterDid length throws.")]
        public async Task BuildCredDefRequestThrows()
        {
            //Arrange 
            string testSubmitterDid = "InvalidLength";
            string testCredDef = "{\"id\":\"testCredDefId\",\"schemaId\":\"testSchemaId\",\"type\":\"CL\",\"tag\":\"\",\"value\":{\"primary\":\"testPrimaryCredentialPublicKey\",\"revoation\":\"\"},\"ver\":\"1.0\"}";

            //Act
            Func<Task> func = async () => await LedgerApi.BuildCredDefRequest(
                testSubmitterDid,
                testCredDef);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion

        #region Tests for BuildCustomRequest
        [Test, TestCase(TestName = "BuildCustomRequest() call returns request handle.")]
        public async Task BuildCustomRequestWorks()
        {
            //Arrange 
            string testRequestJson = "{\"operation\":{\"data\": 1,\"ledgerId\": 1,\"type\": \"3\"},\"protocolVersion\": 2,\"reqId\": 123,\"identifier\": \"LibindyDid111111111111\"}";

            //Act
            IntPtr testObject = await LedgerApi.BuildCustomRequest(
                testRequestJson);

            //Assert
            _ = testObject.Should().NotBe(new IntPtr());
        }

        [Test, TestCase(TestName = "BuildCustomRequest() call with invalid request JSON throws.")]
        public async Task BuildCustomRequestThrows()
        {
            //Arrange 
            string testRequestJson = "{\"invalidJSON\": \"invalid\"}";

            //Act
            Func<Task> func = async () => await LedgerApi.BuildCustomRequest(
                testRequestJson);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion

        #region Tests for BuildDisableAllTxnAuthroAgreementsRequest
        [Test, TestCase(TestName = "BuildDisableAllTxnAuthorAgreementsRequest() call returns request handle.")]
        public async Task BuildDisableAllTxnAuthorAgreementsRequestWorks()
        {
            //Arrange 
            string testSubmitterDid = "LibindyDid111111111111";

            //Act
            IntPtr testObject = await LedgerApi.BuildDisableAllTxnAuthorAgreementsRequest(
                testSubmitterDid);

            //Assert
            _ = testObject.Should().NotBe(new IntPtr());
        }

        [Test, TestCase(TestName = "BuildDisableAllTxnAuthorAgreementsRequest() call with invalid submitterDid length throws.")]
        public async Task BuildDisableAllTxnAuthorAgreementsRequestThrows()
        {
            //Arrange 
            string testSubmitterDid = "InvalidLength";

            //Act
            Func<Task> func = async () => await LedgerApi.BuildDisableAllTxnAuthorAgreementsRequest(
                testSubmitterDid);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion

        #region Tests for BuildGetCredDefRequest
        [Test, TestCase(TestName = "BuildGetCredDefRequest() call returns request handle.")]
        public async Task BuildGetCredDefRequestWorks()
        {
            //Arrange 
            string testCredDefDid = "A9Rsuu7FNquw8Ne2Smu5Nr:3:CL:15:tag";
            string testSubmitterDid = "LibindyDid111111111111";

            //Act
            IntPtr testObject = await LedgerApi.BuildGetCredDefRequest(
                testCredDefDid,
                testSubmitterDid);

            //Assert
            _ = testObject.Should().NotBe(new IntPtr());
        }

        [Test, TestCase(TestName = "BuildGetCredDefRequest() call with invalid submitterDid length throws.")]
        public async Task BuildGetCredDefRequestThrows()
        {
            //Arrange 
            string testCredDefDid = "A9Rsuu7FNquw8Ne2Smu5Nr:3:CL:15:tag";
            string testSubmitterDid = "InvalidLength";

            //Act
            Func<Task> func = async () => await LedgerApi.BuildGetCredDefRequest(
                testCredDefDid,
                testSubmitterDid);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion

        #region Tests for BuildGetNymRequest
        [Test, TestCase(TestName = "BuildGetNymRequest() call returns request handle.")]
        public async Task BuildGetNymRequestWorks()
        {
            //Arrange 
            string testTargetDid = "LibindyDid111111111111";
            string testSubmitterDid = "LibindyDid111111111111";

            //Act
            IntPtr testObject = await LedgerApi.BuildGetNymRequest(
                testTargetDid,
                testSubmitterDid);

            //Assert
            _ = testObject.Should().NotBe(new IntPtr());
        }

        [Test, TestCase(TestName = "BuildGetNymRequest() call with invalid submitterDid length throws.")]
        public async Task BuildGetNymRequestThrows()
        {
            //Arrange 
            string testTargetDid = "LibindyDid111111111111";
            string testSubmitterDid = "InvalidLength";

            //Act
            Func<Task> func = async () => await LedgerApi.BuildGetNymRequest(
                testTargetDid,
                testSubmitterDid);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion

        #region Tests for BuildGetRevocRregDefRequest
        [Test, TestCase(TestName = "BuildGetRevocRegDefRequest() call returns request handle.")]
        public async Task BuildGetRevocRegDefRequestWorks()
        {
            //Arrange 
            string testRevocRegId = "L5wx9FUxCDpFJEdFc23jcn:4:L5wx9FUxCDpFJEdFc23jcn:3:CL:1954:";
            string testSubmitterDid = "LibindyDid111111111111";

            //Act
            IntPtr testObject = await LedgerApi.BuildGetRevocRegDefRequest(
                testRevocRegId,
                testSubmitterDid);

            //Assert
            _ = testObject.Should().NotBe(new IntPtr());
        }

        [Test, TestCase(TestName = "BuildGetRevocRegDefRequest() call with invalid submitterDid length throws.")]
        public async Task BuildGetRevocRegDefRequestThrows()
        {
            //Arrange 
            string testRevocRegId = "L5wx9FUxCDpFJEdFc23jcn:4:L5wx9FUxCDpFJEdFc23jcn:3:CL:1954:";
            string testSubmitterDid = "InvalidLength";

            //Act
            Func<Task> func = async () => await LedgerApi.BuildGetRevocRegDefRequest(
                testRevocRegId,
                testSubmitterDid);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion

        #region Tests for BuildGetRevocRegRequest
        [Test, TestCase(TestName = "BuildGetRevocRegRequest() call returns request handle.")]
        public async Task BuildGetRevocRegRequestWorks()
        {
            //Arrange 
            string testRevocRegId = "L5wx9FUxCDpFJEdFc23jcn:4:L5wx9FUxCDpFJEdFc23jcn:3:CL:1954:";
            long testTimestamp = DateTimeOffset.Now.ToUnixTimeSeconds();
            string testSubmitterDid = "LibindyDid111111111111";

            //Act
            IntPtr testObject = await LedgerApi.BuildGetRevocRegRequest(
                testRevocRegId,
                testTimestamp,
                testSubmitterDid);

            //Assert
            _ = testObject.Should().NotBe(new IntPtr());
        }

        [Test, TestCase(TestName = "BuildGetRevocRegRequest() call with invalid submitterDid length throws.")]
        public async Task BuildGetRevocRegRequestThrows()
        {
            //Arrange 
            string testRevocRegId = "L5wx9FUxCDpFJEdFc23jcn:4:L5wx9FUxCDpFJEdFc23jcn:3:CL:1954:";
            long testTimestamp = DateTimeOffset.Now.ToUnixTimeSeconds();
            string testSubmitterDid = "InvalidLength";

            //Act
            Func<Task> func = async () => await LedgerApi.BuildGetRevocRegRequest(
                testRevocRegId,
                testTimestamp,
                testSubmitterDid);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion

        #region Tests for BuildGetRevocRegDeltaRequestAsync
        [Test, TestCase(TestName = "BuildGetRevocRegDeltaRequestAsync() call returns request handle.")]
        public async Task BuildGetRevocRegDeltaRequestAsyncWorks()
        {
            //Arrange
            string testRevocRegId = "revreg:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:4:creddef:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:3:CL:schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag:CL_ACCUM:TAG_1";
            long testFromTs = 1;
            long testToTs = 1;
            string testSubmitterDid = "LibindyDid111111111111";

            //Act
            IntPtr testObject = await LedgerApi.BuildGetRevocRegDeltaRequestAsync(
                testRevocRegId,
                testToTs,
                testFromTs,
                testSubmitterDid);

            //Assert
            _ = testObject.Should().NotBe(new IntPtr());
        }

        [Test, TestCase(TestName = "BuildGetRevocRegDeltaRequestAsync() call with invalid submitterDid length throws.")]
        public async Task BuildGetRevocRegDeltaRequestAsyncThrows()
        {
            //Arrange
            string testRevocRegId = "revreg:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:4:creddef:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:3:CL:schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag:CL_ACCUM:TAG_1";
            long testFromTs = 1;
            long testToTs = 1;
            string testSubmitterDid = "InvalidLength";

            //Act
            Func<Task> func = async () => await LedgerApi.BuildGetRevocRegDeltaRequestAsync(
                testRevocRegId,
                testToTs,
                testFromTs,
                testSubmitterDid);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion

        #region Tests for BuildGetSchemaRequestAsync
        [Test, TestCase(TestName = "BuildGetSchemaRequestAsync() call returns request handle.")]
        public async Task BuildGetSchemaRequestAsyncWorks()
        {
            //Arrange
            string testSchemaId = "NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0";
            string testSubmitterDid = "LibindyDid111111111111";

            //Act
            IntPtr testObject = await LedgerApi.BuildGetSchemaRequestAsync(
                testSchemaId,
                testSubmitterDid);

            //Assert
            _ = testObject.Should().NotBe(new IntPtr());
        }

        [Test, TestCase(TestName = "BuildGetSchemaRequestAsync() call with invalid submitterDid length throws.")]
        public async Task BuildGetSchemaRequestAsyncThrows()
        {
            //Arrange
            string testSchemaId = "NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0";
            string testSubmitterDid = "InvalidLength";

            //Act
            Func<Task> func = async () => await LedgerApi.BuildGetSchemaRequestAsync(
                testSchemaId,
                testSubmitterDid);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion

        #region Tests for BuildGetTxnAuthorAgreementRequestAsync
        [Test, TestCase(TestName = "BuildGetTxnAuthorAgreementRequestAsync() call returns request handle.")]
        public async Task BuildGetTxnAuthorAgreementRequestAsyncWorks()
        {
            //Arrange
            string testSubmitterDid = "LibindyDid111111111111";
            string data = "{}";

            //Act
            IntPtr testObject = await LedgerApi.BuildGetTxnAuthorAgreementRequestAsync(
                testSubmitterDid,
                data);

            //Assert
            _ = testObject.Should().NotBe(new IntPtr());
        }

        [Test, TestCase(TestName = "BuildGetTxnAuthorAgreementRequestAsync() call with invalid submitterDid length throws.")]
        public async Task BuildGetTxnAuthorAgreementRequestAsyncThrows()
        {
            //Arrange
            string testSubmitterDid = "InvalidLength";
            string data = "{}";

            //Act
            Func<Task> func = async () => await LedgerApi.BuildGetTxnAuthorAgreementRequestAsync(
                testSubmitterDid,
                data);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion

        #region Tests for BuildGetTxnRequestAsync
        [Test, TestCase(TestName = "BuildGetTxnRequestAsync() call returns request handle.")]
        public async Task BuildGetTxnRequestAsyncWorks()
        {
            //Arrange
            int ledgerType = 1;
            int seqNo = 1;
            string testSubmitterDid = "LibindyDid111111111111";

            //Act
            IntPtr testObject = await LedgerApi.BuildGetTxnRequestAsync(
                ledgerType,
                seqNo,
                testSubmitterDid);

            //Assert
            _ = testObject.Should().NotBe(new IntPtr());
        }

        [Test, TestCase(TestName = "BuildGetTxnRequestAsync() call with invalid submitterDid length throws.")]
        public async Task BuildGetTxnRequestAsyncThrows()
        {
            //Arrange
            int ledgerType = 1;
            int seqNo = 1;
            string testSubmitterDid = "InvalidLength";

            //Act
            Func<Task> func = async () => await LedgerApi.BuildGetTxnRequestAsync(
                ledgerType,
                seqNo,
                testSubmitterDid);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion

        #region Tests for BuildGetValidatorInfoRequestAsync
        [Test, TestCase(TestName = "BuildGetValidatorInfoRequestAsync() call returns request handle.")]
        public async Task BuildGetValidatorInfoRequestAsyncWorks()
        {
            //Arrange
            string testSubmitterDid = "LibindyDid111111111111";

            //Act
            IntPtr testObject = await LedgerApi.BuildGetValidatorInfoRequestAsync(
                testSubmitterDid);

            //Assert
            _ = testObject.Should().NotBe(new IntPtr());
        }

        [Test, TestCase(TestName = "BuildGetValidatorInfoRequestAsync() call with invalid submitterDid length throws.")]
        public async Task BuildGetValidatorInfoRequestAsyncThrows()
        {
            //Arrange
            string testSubmitterDid = "InvalidLength";

            //Act
            Func<Task> func = async () => await LedgerApi.BuildGetValidatorInfoRequestAsync(
                testSubmitterDid);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion

        #region Tests for BuildNymRequestAsync
        [Test, TestCase(TestName = "BuildNymRequestAsync() call returns request handle.")]
        public async Task BuildNymRequestAsyncWorks()
        {
            //Arrange
            string testSubmitterDid = "LibindyDid111111111111";
            string dest = "LibindyDid111111111111";
            string verkey = "testVerkey";
            string alias = "testAlias";
            string role = "TRUSTEE";

            //Act
            IntPtr testObject = await LedgerApi.BuildNymRequestAsync(
                testSubmitterDid,
                dest,
                verkey,
                alias,
                role);

            //Assert
            _ = testObject.Should().NotBe(new IntPtr());
        }

        [Test, TestCase(TestName = "BuildNymRequestAsync() call with invalid submitterDid length throws.")]
        public async Task BuildNymRequestAsyncThrows()
        {
            //Arrange
            string testSubmitterDid = "InvalidLength";
            string dest = "LibindyDid111111111111";
            string verkey = "testVerkey";
            string alias = "testAlias";
            string role = "TRUSTEE";

            //Act
            Func<Task> func = async () => await LedgerApi.BuildNymRequestAsync(
                testSubmitterDid,
                dest,
                verkey,
                alias,
                role);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion

        #region Tests for BuildRevocRegDefRequestAsync
        [Test, TestCase(TestName = "BuildRevocRegDefRequestAsync() call returns request handle.")]
        public async Task BuildRevocRegDefRequestAsyncWorks()
        {
            //Arrange
            string testSubmitterDid = "LibindyDid111111111111";

            string revRegDefJson = "{\"id\":\"testId\",\"revocDefType\":\"CL_ACCUM\",\"tag\":\"testTag\",\"credDefId\":\"testCredDefId\",\"value\":{\"issuanceType\":\"ISSUANCE_BY_DEFAULT\",\"maxCredNum\":5,\"publicKeys\":{\"accumKey\":\"testAccumKey\"},\"tailsHash\":\"testTailsHash\",\"tailsLocation\":\"testTailsLocation\"},\"ver\":\"1.0\"}";

            //Act
            IntPtr testObject = await LedgerApi.BuildRevocRegDefRequestAsync(
                testSubmitterDid,
                revRegDefJson);

            //Assert
            _ = testObject.Should().NotBe(new IntPtr());
        }

        [Test, TestCase(TestName = "BuildRevocRegDefRequestAsync() call with invalid submitterDid length throws.")]
        public async Task BuildRevocRegDefRequestAsyncThrows()
        {
            //Arrange
            string testSubmitterDid = "InvalidLength";

            string revRegDefJson = "{\"id\":\"testId\",\"revocDefType\":\"CL_ACCUM\",\"tag\":\"testTag\",\"credDefId\":\"testCredDefId\",\"value\":{\"issuanceType\":\"ISSUANCE_BY_DEFAULT\",\"maxCredNum\":5,\"publicKeys\":{\"accumKey\":\"testAccumKey\"},\"tailsHash\":\"testTailsHash\",\"tailsLocation\":\"testTailsLocation\"},\"ver\":\"1.0\"}";

            //Act
            Func<Task> func = async () => await LedgerApi.BuildRevocRegDefRequestAsync(
                testSubmitterDid,
                revRegDefJson);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion

        #region Tests for BuildRevocRegEntryRequestAsync
        [Test, TestCase(TestName = "BuildRevocRegEntryRequestAsync() call returns request handle.")]
        public async Task BuildRevocRegEntryRequestAsyncWorks()
        {
            //Arrange
            string testSubmitterDid = "LibindyDid111111111111";
            string testRevRegDefId = "revreg:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:4:creddef:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:3:CL:schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag:CL_ACCUM:TAG_1";
            string testRevRegDefType = "CL_ACCUM";

            string deltaJson = "{\"ver\":\"1.0\",\"value\":\"test\"}";

            //Act
            IntPtr testObject = await LedgerApi.BuildRevocRegEntryRequestAsync(
                testSubmitterDid,
                testRevRegDefId,
                testRevRegDefType,
                deltaJson);

            //Assert
            _ = testObject.Should().NotBe(new IntPtr());
        }

        [Test, TestCase(TestName = "BuildRevocRegEntryRequestAsync() call with invalid submitterDid length throws.")]
        public async Task BuildRevocRegEntryRequestAsyncThrows()
        {
            //Arrange
            string testSubmitterDid = "InvalidLength";
            string testRevRegDefId = "revreg:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:4:creddef:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:3:CL:schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag:CL_ACCUM:TAG_1";
            string testRevRegDefType = "CL_ACCUM";

            string deltaJson = "{\"ver\":\"1.0\",\"value\":\"test\"}";

            //Act
            Func<Task> func = async () => await LedgerApi.BuildRevocRegEntryRequestAsync(
                testSubmitterDid,
                testRevRegDefId,
                testRevRegDefType,
                deltaJson);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion

        #region Tests for BuildSchemaAsync
        [Test, TestCase(TestName = "BuildSchemaRequestAsync() call returns request handle.")]
        public async Task BuildSchemaRequestAsyncWorks()
        {
            //Arrange
            string testSubmitterDid = "LibindyDid111111111111";

            string schemaJson = "{\"Handle\":0,\"id\":\"testId\",\"name\":\"testName\",\"version\":\"1.0\",\"attrNames\":[\"testAttribute1\",\"testAttribute2\"],\"ver\":\"1.0\",\"SeqNo\":5}";

            //Act
            IntPtr testObject = await LedgerApi.BuildSchemaRequestAsync(
                testSubmitterDid,
                schemaJson);

            //Assert
            _ = testObject.Should().NotBe(new IntPtr());
        }

        [Test, TestCase(TestName = "BuildSchemaRequestAsync() call with invalid submitterDid length throws.")]
        public async Task BuildSchemaRequestAsyncThrows()
        {
            //Arrange
            string testSubmitterDid = "InvalidLength";

            string schemaJson = "{\"Handle\":0,\"id\":\"testId\",\"name\":\"testName\",\"version\":\"1.0\",\"attrNames\":[\"testAttribute1\",\"testAttribute2\"],\"ver\":\"1.0\",\"SeqNo\":5}";

            //Act
            Func<Task> func = async () => await LedgerApi.BuildSchemaRequestAsync(
                testSubmitterDid,
                schemaJson);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion

        #region Tests for BuildTxnAuthorAgreementRequestAsync
        [Test, TestCase(TestName = "BuildTxnAuthorAgreementRequestAsync() call returns request handle.")]
        public async Task BuildTxnAuthorAgreementRequestAsyncWorks()
        {
            //Arrange
            string testSubmitterDid = "LibindyDid111111111111";
            string testText = "Text";
            string testVersion = "1.0";
            long testRatificationTs = 1;
            long testRetirementTs = 1;

            //Act
            IntPtr testObject = await LedgerApi.BuildTxnAuthorAgreementRequestAsync(
                testSubmitterDid,
                testText,
                testVersion,
                testRatificationTs,
                testRetirementTs);

            //Assert
            _ = testObject.Should().NotBe(new IntPtr());
        }

        [Test, TestCase(TestName = "BuildTxnAuthorAgreementRequestAsync() call with invalid submitterDid throws.")]
        public async Task BuildTxnAuthorAgreementRequestAsyncThrows()
        {
            //Arrange
            string testSubmitterDid = "InvalidLength";
            string testText = "Text";
            string testVersion = "1.0";
            long testRatificationTs = 1;
            long testRetirementTs = 1;

            //Act
            Func<Task> func = async () => await LedgerApi.BuildTxnAuthorAgreementRequestAsync(
                testSubmitterDid,
                testText,
                testVersion,
                testRatificationTs,
                testRetirementTs);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion

        #region Parse methods
        [Test, TestCase(TestName = "ParseGetSchemaResponse() takes a response JSON and parses it to a schema JSON.")]
        public async Task ParseGetSchemaResponseWorks()
        {
            //Arrange
            IntPtr testPoolHandle = await PoolApi.CreatePoolAsync(null, _genesisFilePath, null);
            IntPtr testRequestHandle = await LedgerApi.BuildGetSchemaRequestAsync("9vBvpoNHmqiDu4pAUVVue7:2:Boarding Pass:1.0");
            string response = await PoolApi.SubmitPoolRequestAsync(testPoolHandle, testRequestHandle);
            //Act
            string res = LedgerApi.ParseGetSchemaResponse(response);

            //Assert
            var resJObj = JObject.Parse(res);

            string version = resJObj["version"].ToString();
            string id = resJObj["id"].ToString();

            version.Should().NotBeNullOrEmpty();
            id.Should().NotBeNullOrEmpty();
        }

        [Test, TestCase(TestName = "ParseGetCredDefResponse() takes a response JSON and parses it to a schema JSON.")]
        public async Task ParseGetCredDefResponseWorks()
        {
            //Arrange
            IntPtr testPoolHandle = await PoolApi.CreatePoolAsync(null, _genesisFilePath, null);
            string testCredDefDid = "J8L3C8nBZxSRVu1DueRbJR:3:CL:52019:9491087";
            string testSubmitterDid = "LibindyDid111111111111";

            IntPtr testRequestHandle = await LedgerApi.BuildGetCredDefRequest(testCredDefDid, testSubmitterDid);
                        
            string response = await PoolApi.SubmitPoolRequestAsync(testPoolHandle, testRequestHandle);

            //Act
            string res = LedgerApi.ParseGetCredDefResponse(response);

            //Assert
            var resJObj = JObject.Parse(res);

            string ver = resJObj["ver"].ToString();
            string id = resJObj["id"].ToString();
            string z = JObject.Parse(res)["value"]["primary"]["z"].ToString();
          
            ver.Should().NotBeNullOrEmpty();
            id.Should().NotBeNullOrEmpty();
            z.Should().NotBeNullOrEmpty();
        }

        [Test, TestCase(TestName = "ParseGetRevocRegDefResponse() takes a response JSON and parses it to a schema JSON.")]
        public async Task ParseGetRevocRegDefResponseWorks()
        {
            //Arrange
            IntPtr testPoolHandle = await PoolApi.CreatePoolAsync(null, _genesisFilePath, null);
            string testRevocRegId = "4o469o3tnHVKpcTUhK8T6Y:4:4o469o3tnHVKpcTUhK8T6Y:3:CL:53194:default:CL_ACCUM:e141ed22-65e4-4d2d-85bd-66729e9b97cc";
            string testSubmitterDid = "LibindyDid111111111111";
            IntPtr testRequestHandle = await LedgerApi.BuildGetRevocRegDefRequest(
                testRevocRegId,
                testSubmitterDid);
            string response = await PoolApi.SubmitPoolRequestAsync(testPoolHandle, testRequestHandle);
            //Act
            string res = LedgerApi.ParseGetRevocRegDefResponseAsync(response);

            //Assert
            var resJObj = JObject.Parse(res);

            string ver = resJObj["ver"].ToString();
            string id = resJObj["id"].ToString();
            string revocDefType = resJObj["revocDefType"].ToString();
            string issuanceType = resJObj["value"]["issuanceType"].ToString();
            string z = resJObj["value"]["publicKeys"]["accumKey"]["z"].ToString();

            ver.Should().NotBeNullOrEmpty();
            id.Should().NotBeNullOrEmpty();
            revocDefType.Should().NotBeNullOrEmpty();
            issuanceType.Should().NotBeNullOrEmpty();
            z.Should().NotBeNullOrEmpty();
        }

        [Test, TestCase(TestName = "ParseGetRevocRegResponse() takes a response JSON and parses it to a schema JSON.")]
        public async Task ParseGetRevocRegResponseWorks()
        {
            //Arrange
            IntPtr testPoolHandle = await PoolApi.CreatePoolAsync(null, _genesisFilePath, null);
            string testRevocRegId = "VH5KLUSKxFAuS9mbXQEJCK:4:VH5KLUSKxFAuS9mbXQEJCK:3:CL:53251:postMan-Shoes:CL_ACCUM:8d89d381-30a6-4185-9a76-018019f20ad5";
            long testTimestamp = DateTimeOffset.Now.ToUnixTimeSeconds();
            string testSubmitterDid = "LibindyDid111111111111";

            //Act
            IntPtr testRequestHandle = await LedgerApi.BuildGetRevocRegRequest(
                testRevocRegId,
                testTimestamp,
                testSubmitterDid);
            string response = await PoolApi.SubmitPoolRequestAsync(testPoolHandle, testRequestHandle);
            //Act
            string res = LedgerApi.ParseGetRevocRegResponseAsync(response);

            //Assert
            var resJObj = JObject.Parse(res);

            string ver = resJObj["ver"].ToString();
            string accum = resJObj["value"]["accum"].ToString();

            ver.Should().NotBeNullOrEmpty();
            accum.Should().NotBeNullOrEmpty();
        }


        #endregion
    }
}
