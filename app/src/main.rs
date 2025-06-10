use ethereum::{AccessList, EIP1559Transaction, EIP1559TransactionMessage, TransactionAction};
use ethereum_types::H256;
use k256::ecdsa::SigningKey;

mod config;
mod de;
mod error;
mod params;

type Result<T> = std::result::Result<T, error::Error>;

fn main() -> Result<()> {
    dotenv::dotenv()?;

    // 環境変数で渡される設定値
    let config = crate::config::Config::from_env()?;

    // パラメータJSONをパース
    let params_json_path = std::env::args()
        .nth(1)
        .expect("Missing argument: Please provide the path to parameter json file.");
    let params = params::Params::from_path(params_json_path);

    // 署名値 (odd_y_parity, r, s) を含まないトランザクションデータを作成
    let transaction_message = EIP1559TransactionMessage {
        chain_id: config.chain_id,
        nonce: params.nonce,
        max_priority_fee_per_gas: config.max_priority_fee_per_gas,
        max_fee_per_gas: config.max_fee_per_gas,
        gas_limit: params.gas_limit,
        action: TransactionAction::Call(params.to_address),
        value: params.value,
        input: params.input,
        access_list: AccessList::default(),
    };

    // 署名用ハッシュを計算
    let transaction_hash = transaction_message.hash();

    // 秘密鍵
    let private_key_bytes = config.get_private_key_bytes()?;
    // 秘密鍵から SigningKey 作成
    let signing_key = SigningKey::from_slice(&private_key_bytes)?;

    // 署名とrecovery_idを同時に取得
    let (signature, recovery_id) = signing_key.sign_prehash_recoverable(&transaction_hash.0)?;

    // トランザクションデータを作成
    let (r_bytes, s_bytes) = signature.split_bytes();
    let transaction = EIP1559Transaction {
        chain_id: transaction_message.chain_id,
        nonce: transaction_message.nonce,
        max_priority_fee_per_gas: transaction_message.max_priority_fee_per_gas,
        max_fee_per_gas: transaction_message.max_fee_per_gas,
        gas_limit: transaction_message.gas_limit,
        action: transaction_message.action,
        value: transaction_message.value,
        input: transaction_message.input,
        access_list: transaction_message.access_list,
        odd_y_parity: (recovery_id.to_byte() & 1) == 1, // recovery_id が奇数かどうかを判定
        r: H256::from_slice(&r_bytes),
        s: H256::from_slice(&s_bytes),
    };

    // 署名済みトランザクションをRLPエンコード
    let rlp_encoded_transaction_bytes = rlp::encode(&transaction);

    // Type 2 プレフィックス付きの最終形式
    let signed_transaction = {
        let mut buf = vec![0x02];
        buf.extend_from_slice(&rlp_encoded_transaction_bytes);
        buf
    };

    // 16進数文字列として出力
    println!("0x{}", hex::encode(signed_transaction));

    Ok(())
}
