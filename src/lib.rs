use metashrew_support::index_pointer::KeyValuePointer;
use metashrew_support::compat::to_arraybuffer_layout;

use alkanes_runtime::{
  declare_alkane, message::MessageDispatch, storage::StoragePointer, token::Token,
  runtime::AlkaneResponder
};

use alkanes_support::{
  cellpack::Cellpack, id::AlkaneId,
  parcel::{AlkaneTransfer, AlkaneTransferParcel}, response::CallResponse
};

use anyhow::{anyhow, Result};
use std::sync::Arc;

mod svg_generator;
use svg_generator::SvgGenerator;

/// Orbital template ID
const DOGI_ORBITAL_TEMPLATE_ID: u128 = 0x378;

#[derive(Default)]
pub struct GigaDogiCollection(());

impl AlkaneResponder for GigaDogiCollection {}

#[derive(MessageDispatch)]
enum GigaDogiCollectionMessage {
  #[opcode(0)]
  Initialize,

  #[opcode(69)]
  AuthMintOrbital { count: u128 },

  #[opcode(77)]
  MintOrbital,

  #[opcode(99)]
  #[returns(String)]
  GetName,

  #[opcode(100)]
  #[returns(String)]
  GetSymbol,

  #[opcode(101)]
  #[returns(u128)]
  GetTotalSupply,

  #[opcode(102)]
  #[returns(u128)]
  GetOrbitalCount,

  #[opcode(999)]
  #[returns(String)]
  GetAttributes { index: u128 },

  #[opcode(1000)]
  #[returns(Vec<u8>)]
  GetData { index: u128 },

  #[opcode(1001)]
  #[returns(Vec<u8>)]
  GetInstanceAlkaneId { index: u128 },

  #[opcode(1002)]
  #[returns(String)]
  GetInstanceIdentifier { index: u128 },
}

impl Token for GigaDogiCollection {
  fn name(&self) -> String {
    return String::from("Giga Dogi")
  }

  fn symbol(&self) -> String {
    return String::from("giga-dogi");
  }
}

impl GigaDogiCollection {
  fn initialize(&self) -> Result<CallResponse> {
    self.observe_initialization()?;
    let context = self.context()?;

    let mut response = CallResponse::forward(&context.incoming_alkanes);

    // Collection token acts as auth token for contract minting without any limits
    response.alkanes.0.push(AlkaneTransfer {
      id: context.myself.clone(),
      value: 10u128,
    });

    Ok(response)
  }

  fn auth_mint_orbital(&self, count: u128) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    // Authorized mints
    self.only_owner()?;

    let mut minted_orbitals = Vec::new();

    for _ in 0..count {
      minted_orbitals.push(self.create_mint_transfer()?);
    }

    response.alkanes.0.extend(minted_orbitals);

    Ok(response)
  }

  fn mint_orbital(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    // Allow minting on all blocks (removed block height restrictions)
    response.alkanes.0.push(self.create_mint_transfer()?);

    Ok(response)
  }

  fn create_mint_transfer(&self) -> Result<AlkaneTransfer> {
    let index = self.instances_count();

    if index >= self.max_mints() {
      return Err(anyhow!("Giga Dogi collection has fully minted out"));
    }

    let cellpack = Cellpack {
      target: AlkaneId {
        block: 6,
        tx: DOGI_ORBITAL_TEMPLATE_ID,
      },
      inputs: vec![0x0, index],
    };

    let sequence = self.sequence();
    let response = self.call(&cellpack, &AlkaneTransferParcel::default(), self.fuel())?;

    let orbital_id = AlkaneId {
      block: 2,
      tx: sequence,
    };

    self.add_instance(&orbital_id)?;

    if response.alkanes.0.len() < 1 {
      Err(anyhow!("orbital token not returned with factory"))
    } else {
      Ok(response.alkanes.0[0])
    }
  }

  fn max_mints(&self) -> u128 {
    5 // Supply max de 5 NFTs
  }

  fn max_mint_per_block(&self) -> u32 {
    5 // Permet de mint jusqu'à 5 par bloc (toute la collection d'un coup si désiré)
  }

  // Fonction observe_mint supprimée pour permettre le mint sur tous les blocs
  // sans restrictions de hauteur de bloc ou de parité

  fn seen_pointer(&self, hash: &Vec<u8>) -> StoragePointer {
    StoragePointer::from_keyword("/seen/").select(&hash)
  }

  fn get_name(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    response.data = self.name().into_bytes();

    Ok(response)
  }

  fn get_symbol(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    response.data = self.symbol().into_bytes();

    Ok(response)
  }

  fn get_total_supply(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    response.data = 1u128.to_le_bytes().to_vec();

    Ok(response)
  }

  fn get_orbital_count(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    response.data = self.instances_count().to_le_bytes().to_vec();

    Ok(response)
  }

  fn get_attributes(&self, index: u128) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    // Génère les attributs pour les 5 Giga Dogi avec 5 traits chacun
    let attributes = self.generate_dogi_attributes(index)?;
    response.data = attributes.into_bytes();
    Ok(response)
  }

  fn get_data(&self, index: u128) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    // Retourne l'URL Cloudinary pour l'image du Dogi
    let cloudinary_url = self.get_cloudinary_url(index)?;
    response.data = cloudinary_url.into_bytes();
    Ok(response)
  }

  fn generate_dogi_attributes(&self, index: u128) -> Result<String> {
    if index >= 5 {
      return Err(anyhow!("Index out of bounds for Giga Dogi collection"));
    }

    // Définit les traits pour chaque Dogi (5 traits par NFT)
    let dogi_traits = match index {
      0 => r#"{"attributes": [
        {"trait_type": "Background", "value": "Cosmic Purple"},
        {"trait_type": "Body", "value": "Golden Shiba"},
        {"trait_type": "Eyes", "value": "Laser Blue"},
        {"trait_type": "Accessory", "value": "Diamond Chain"},
        {"trait_type": "Rarity", "value": "Legendary"}
      ]}"#,
      1 => r#"{"attributes": [
        {"trait_type": "Background", "value": "Neon Green"},
        {"trait_type": "Body", "value": "Silver Shiba"},
        {"trait_type": "Eyes", "value": "Fire Red"},
        {"trait_type": "Accessory", "value": "Bitcoin Crown"},
        {"trait_type": "Rarity", "value": "Epic"}
      ]}"#,
      2 => r#"{"attributes": [
        {"trait_type": "Background", "value": "Electric Blue"},
        {"trait_type": "Body", "value": "Rainbow Shiba"},
        {"trait_type": "Eyes", "value": "Galaxy Purple"},
        {"trait_type": "Accessory", "value": "Rocket Pack"},
        {"trait_type": "Rarity", "value": "Rare"}
      ]}"#,
      3 => r#"{"attributes": [
        {"trait_type": "Background", "value": "Sunset Orange"},
        {"trait_type": "Body", "value": "Cyber Shiba"},
        {"trait_type": "Eyes", "value": "Neon Green"},
        {"trait_type": "Accessory", "value": "Holographic Collar"},
        {"trait_type": "Rarity", "value": "Uncommon"}
      ]}"#,
      4 => r#"{"attributes": [
        {"trait_type": "Background", "value": "Matrix Black"},
        {"trait_type": "Body", "value": "Platinum Shiba"},
        {"trait_type": "Eyes", "value": "Diamond White"},
        {"trait_type": "Accessory", "value": "Infinity Gauntlet"},
        {"trait_type": "Rarity", "value": "Mythic"}
      ]}"#,
      _ => return Err(anyhow!("Invalid Dogi index")),
    };

    Ok(dogi_traits.to_string())
  }

  fn get_cloudinary_url(&self, index: u128) -> Result<String> {
    if index >= 5 {
      return Err(anyhow!("Index out of bounds for Giga Dogi collection"));
    }

    // URLs Cloudinary pour chaque Giga Dogi
    let cloudinary_urls = [
      "https://res.cloudinary.com/dpwvlwwf7/image/upload/t_Thumbnail/v1749684070/dogi5_iig8fx.png",
      "https://res.cloudinary.com/dpwvlwwf7/image/upload/t_Thumbnail/v1749684069/dogi3_bfezed.png", 
      "https://res.cloudinary.com/dpwvlwwf7/image/upload/t_Thumbnail/v1749684069/dogi4_jistot.png",
      "https://res.cloudinary.com/dpwvlwwf7/image/upload/t_Thumbnail/v1749684068/dogi2_gowfmy.png",
      "https://res.cloudinary.com/dpwvlwwf7/image/upload/t_Thumbnail/v1749684067/dogi1_k1xpl4.png"
    ];

    Ok(cloudinary_urls[index as usize].to_string())
  }

  fn instances_pointer(&self) -> StoragePointer {
    StoragePointer::from_keyword("/instances")
  }

  fn instances_count(&self) -> u128 {
    self.instances_pointer().get_value::<u128>()
  }

  fn set_instances_count(&self, count: u128) {
    self.instances_pointer().set_value::<u128>(count);
  }

  fn add_instance(&self, instance_id: &AlkaneId) -> Result<u128> {
    let count = self.instances_count();
    let new_count = count.checked_add(1)
      .ok_or_else(|| anyhow!("instances count overflow"))?;

    let mut bytes = Vec::with_capacity(32);
    bytes.extend_from_slice(&instance_id.block.to_le_bytes());
    bytes.extend_from_slice(&instance_id.tx.to_le_bytes());

    let bytes_vec = new_count.to_le_bytes().to_vec();
    let mut instance_pointer = self.instances_pointer().select(&bytes_vec);
    instance_pointer.set(Arc::new(bytes));
    
    self.set_instances_count(new_count);
    
    Ok(new_count)
  }

  fn only_owner(&self) -> Result<()> {
    let context = self.context()?;

    if context.incoming_alkanes.0.len() != 1 {
      return Err(anyhow!(
        "did not authenticate with only the collection token"
      ));
    }

    let transfer = context.incoming_alkanes.0[0].clone();
    if transfer.id != context.myself.clone() {
      return Err(anyhow!("supplied alkane is not collection token"));
    }

    if transfer.value < 1 {
      return Err(anyhow!(
        "less than 1 unit of collection token supplied to authenticate"
      ));
    }

    Ok(())
  }

  fn lookup_instance(&self, index: u128) -> Result<AlkaneId> {
    // Add 1 to index since instances are stored at 1-based indices
    let storage_index = index + 1;
    let bytes_vec = storage_index.to_le_bytes().to_vec();
    
    let instance_pointer = self.instances_pointer().select(&bytes_vec);
    
    let bytes = instance_pointer.get();
    if bytes.len() != 32 {
      return Err(anyhow!("Invalid instance data length"));
    }

    let block_bytes = &bytes[..16];
    let tx_bytes = &bytes[16..];

    let block = u128::from_le_bytes(block_bytes.try_into().unwrap());
    let tx = u128::from_le_bytes(tx_bytes.try_into().unwrap());

    Ok(AlkaneId { block, tx })
  }

  fn get_instance_alkane_id(&self, index: u128) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    let instance_id = self.lookup_instance(index)?;

    let mut bytes = Vec::with_capacity(32);
    bytes.extend_from_slice(&instance_id.block.to_le_bytes());
    bytes.extend_from_slice(&instance_id.tx.to_le_bytes());

    response.data = bytes;
    Ok(response)
  }

  fn get_instance_identifier(&self, index: u128) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    let instance_id = self.lookup_instance(index)?;
    let instance_str = format!("{}:{}", instance_id.block, instance_id.tx);
    
    response.data = instance_str.into_bytes();
    Ok(response)
  }
}

declare_alkane! {
  impl AlkaneResponder for GigaDogiCollection {
    type Message = GigaDogiCollectionMessage;
  }
}