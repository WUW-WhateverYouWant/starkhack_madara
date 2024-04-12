(function() {var implementors = {
"mc_rpc":[["impl&lt;A, B, BE, G, C, P, H&gt; <a class=\"trait\" href=\"mc_rpc/trait.MadaraRpcApiServer.html\" title=\"trait mc_rpc::MadaraRpcApiServer\">MadaraRpcApiServer</a> for <a class=\"struct\" href=\"mc_rpc/struct.Starknet.html\" title=\"struct mc_rpc::Starknet\">Starknet</a>&lt;A, B, BE, G, C, P, H&gt;<span class=\"where fmt-newline\">where\n    A: ChainApi&lt;Block = B&gt; + 'static,\n    B: BlockT,\n    BE: Backend&lt;B&gt; + 'static,\n    C: HeaderBackend&lt;B&gt; + BlockBackend&lt;B&gt; + StorageProvider&lt;B, BE&gt; + 'static + ProvideRuntimeApi&lt;B&gt;,\n    G: GenesisProvider + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> + 'static,\n    C::Api: <a class=\"trait\" href=\"pallet_starknet_runtime_api/trait.StarknetRuntimeApi.html\" title=\"trait pallet_starknet_runtime_api::StarknetRuntimeApi\">StarknetRuntimeApi</a>&lt;B&gt; + <a class=\"trait\" href=\"pallet_starknet_runtime_api/trait.ConvertTransactionRuntimeApi.html\" title=\"trait pallet_starknet_runtime_api::ConvertTransactionRuntimeApi\">ConvertTransactionRuntimeApi</a>&lt;B&gt;,\n    P: TransactionPool&lt;Block = B&gt; + 'static,\n    H: <a class=\"trait\" href=\"mp_hashers/trait.HasherT.html\" title=\"trait mp_hashers::HasherT\">HasherT</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> + 'static,</span>"],["impl&lt;A, B, BE, G, C, P, H&gt; <a class=\"trait\" href=\"mc_rpc/trait.MadaraRpcApiServer.html\" title=\"trait mc_rpc::MadaraRpcApiServer\">MadaraRpcApiServer</a> for <a class=\"struct\" href=\"mc_rpc/starknetrpcwrapper/struct.StarknetRpcWrapper.html\" title=\"struct mc_rpc::starknetrpcwrapper::StarknetRpcWrapper\">StarknetRpcWrapper</a>&lt;A, B, BE, G, C, P, H&gt;<span class=\"where fmt-newline\">where\n    A: ChainApi&lt;Block = B&gt; + 'static,\n    B: BlockT,\n    BE: Backend&lt;B&gt; + 'static,\n    C: HeaderBackend&lt;B&gt; + BlockBackend&lt;B&gt; + StorageProvider&lt;B, BE&gt; + 'static + ProvideRuntimeApi&lt;B&gt;,\n    G: GenesisProvider + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> + 'static,\n    C::Api: <a class=\"trait\" href=\"pallet_starknet_runtime_api/trait.StarknetRuntimeApi.html\" title=\"trait pallet_starknet_runtime_api::StarknetRuntimeApi\">StarknetRuntimeApi</a>&lt;B&gt; + <a class=\"trait\" href=\"pallet_starknet_runtime_api/trait.ConvertTransactionRuntimeApi.html\" title=\"trait pallet_starknet_runtime_api::ConvertTransactionRuntimeApi\">ConvertTransactionRuntimeApi</a>&lt;B&gt;,\n    P: TransactionPool&lt;Block = B&gt; + 'static,\n    H: <a class=\"trait\" href=\"mp_hashers/trait.HasherT.html\" title=\"trait mp_hashers::HasherT\">HasherT</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> + 'static,</span>"]]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()