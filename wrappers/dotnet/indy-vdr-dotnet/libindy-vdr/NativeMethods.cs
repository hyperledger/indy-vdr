using System.Runtime.InteropServices;
using static indy_vdr_dotnet.models.Structures;

namespace indy_vdr_dotnet.libindy_vdr
{
    internal static class NativeMethods
    {
        #region Error
        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_get_current_error(ref string error_json_p);
        #endregion

        #region Ledger
        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_build_acceptance_mechanisms_request(FfiStr submitter_did, FfiStr aml, FfiStr version, FfiStr aml_context, ref uint handle_p);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_build_get_acceptance_mechanisms_request(FfiStr submitter_did, long timestamp, FfiStr version, ref uint handle_p);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_build_attrib_request(FfiStr submitter_did, FfiStr target_did, FfiStr hash, FfiStr raw, FfiStr enc, ref uint handle_p);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_build_get_attrib_request(FfiStr submitter_did, FfiStr target_did, FfiStr raw, FfiStr hash, FfiStr enc, ref uint handle_p);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_build_cred_def_request(FfiStr submitter_did, FfiStr cred_def, ref uint handle_p);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_build_custom_request(FfiStr submitter_did, ref uint handle_p);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_build_disable_all_txn_author_agreements_request(FfiStr submitter_did, ref uint handle_p);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_build_get_cred_def_request(FfiStr submitter_did, FfiStr cred_def_id, ref uint handle_p);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_build_get_nym_request(FfiStr submitter_did, FfiStr dest, ref uint handle_p);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_build_get_revoc_reg_def_request(FfiStr submitter_did, FfiStr revoc_reg_id, ref uint handle_p);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_build_get_revoc_reg_request(FfiStr submitter_did, FfiStr revoc_reg_id, long timestamp, ref uint handle_p);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_build_get_revoc_reg_delta_request(FfiStr submitter_did, FfiStr revoc_reg_id, long from_ts, long to_ts, ref uint handle_p);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_build_get_schema_request(FfiStr submitter_did, FfiStr schema_id, ref uint handle_p);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_build_get_txn_author_agreement_request(FfiStr submitter_did, FfiStr data, ref uint handle_p);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_build_get_txn_request(FfiStr submitter_did, int ledger_type, int seq_no, ref uint handle_p);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_build_get_validator_info_request(FfiStr submitter_did, ref uint handle_p);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_build_nym_request(FfiStr submitter_did, FfiStr dest, FfiStr verkey, FfiStr alias, FfiStr role, ref uint handle_p);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_build_revoc_reg_def_request(FfiStr submitter_did, FfiStr revoc_reg_def, ref uint handle_p);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_build_revoc_reg_entry_request(FfiStr submitter_did, FfiStr revoc_reg_def_id, FfiStr revoc_reg_def_type, FfiStr revoc_reg_entry, ref uint handle_p);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_build_schema_request(FfiStr submitter_did, FfiStr schema, ref uint handle_p);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_build_txn_author_agreement_request(FfiStr submitter_did, FfiStr text, FfiStr version, long ratification_ts, long retirement_ts, ref uint handle_p);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_build_rich_schema_request(FfiStr submitter_did, FfiStr rs_id, FfiStr rs_content, FfiStr rs_name, FfiStr rs_version, FfiStr rs_type, FfiStr ver, ref uint handle_p);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_build_get_rich_schema_object_by_id_request(FfiStr submitter_did, FfiStr rs_id, ref uint handle_p);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_build_get_rich_schema_object_by_metadata_request(FfiStr submitter_did, FfiStr rs_type, FfiStr rs_name, FfiStr rs_version, ref uint handle_p);
        #endregion

        #region Mod
        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_set_config(FfiStr config);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_set_default_logger();

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_set_protocol_version(uint version);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_set_socks_proxy(FfiStr socks_proxy);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern string indy_vdr_version();
        #endregion

        #region Pool
        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_pool_create(FfiStr param, ref uint handle_p);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_pool_refresh(uint pool_handle);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_pool_get_status(uint pool_handle);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_pool_get_transactions(uint pool_handle);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_pool_get_verifiers(uint pool_handle);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_pool_submit_action(uint pool_handle, uint request_handle, FfiStr nodes, int timeout);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_pool_submit_request(uint pool_handle, uint request_handle);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_pool_close(uint pool_handle);
        #endregion

        #region Request
        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_prepare_txn_author_agreement_acceptance(FfiStr text, FfiStr version, FfiStr taa_digest, FfiStr acc_mech_type, ulong time, ref string output_p);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_request_free(uint request_handle);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_request_get_body(uint request_handle, ref string body_p);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_request_get_signature_input(uint request_handle, ref string input_p);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_request_set_endorser(uint request_handle, FfiStr endorser);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_request_set_multi_signature(uint request_handle, FfiStr identifier, ByteBuffer signature);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_request_set_signature(uint request_handle, ByteBuffer signature);

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern int indy_vdr_request_set_txn_author_agreement_acceptance(uint request_handle, FfiStr acceptance);
        #endregion
    }
}