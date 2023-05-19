use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

declare_id!("8EEY7nX8xNTgNHvsYAhKZ8TwLP6eJLEhBZGbJWx3vtD6");

#[program]
pub mod versioning {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, args: InitializeArgs) -> Result<()> {
        ctx.accounts.data.set_inner(Data {
            version: Version::V1{ foo: args.foo },
        });

        Ok(())
    }

    pub fn migrate(ctx: Context<Migrate>) -> Result<()> {
        let data_account_info = &ctx.accounts.data.to_account_info();

        let data = &mut ctx.accounts.data;

        let old_space = data.current_space();

        match &mut data.version {
            Version::V1 { foo } => {
                data.version = Version::V2 {
                    foo: *foo,
                    bar: 4545,
                }
            }
            Version::V2 { foo: _, bar: _ } => {
                data.version = Version::V3 { baz: "hello-world-very-long".to_string() }
            }
            _ => {},
        };

        let new_space = data.current_space();

        let diff: i64 = new_space as i64 - old_space as i64;

        reallocate(diff, data_account_info, ctx.accounts.payer.clone(), ctx.accounts.system_program.clone())
    }
}

#[derive(Accounts)]
#[instruction(args: InitializeArgs)]
pub struct Initialize<'info> {
    #[account(init, payer = payer, space = Data::init_space(Version::V1{ foo: args.foo }), seeds = [Data::PREFIX.as_bytes()], bump)]
    pub data: Account<'info, Data>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub rent: Sysvar<'info, Rent>,

    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeArgs {
    pub foo: u64,
}

#[derive(Accounts)]
pub struct Migrate<'info> {
    #[account(mut, seeds = [Data::PREFIX.as_bytes()], bump)]
    pub data: Account<'info, Data>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub rent: Sysvar<'info, Rent>,

    pub system_program: Program<'info, System>,
}

#[account]
pub struct Data {
    pub version: Version,
}

impl Data {
    pub const PREFIX: &'static str = "data";
    pub fn init_space(version: Version) -> usize {
        8 + // anchor
        version.space()
    }

    pub fn current_space(&self) -> usize {
        8 + // anchor
        self.version.space()
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum Version {
    V1 { foo: u64 },
    V2 { foo: u64, bar: u64 },
    V3 { baz: String },
}

impl Version {
    // space of enum is always the largest variant
    pub fn space(&self) -> usize {
        let mut enum_bytes = 1;
        match &self {
            Version::V1 { foo: _ } => {
                enum_bytes += 8
            }
            Version::V2 { foo: _, bar: _ } => {
                enum_bytes += 16;
            }
            Version::V3 { baz } => {
                enum_bytes += 4 + baz.len();
            }
        }

        enum_bytes
    }
}

pub fn reallocate<'info>(
    size_diff: i64,
    account: &AccountInfo<'info>,
    payer: Signer<'info>,
    system_program: Program<'info, System>,
) -> Result<()> {
    let new_size = (account.data_len() as i64 + size_diff) as usize;
    account.realloc(new_size, false)?;

    let rent = Rent::get()?;

    let lamports_required = rent.minimum_balance(new_size);

    let current_lamports = account.lamports();

    let transfer_amount: i64 = (lamports_required as i64) - (current_lamports as i64);

    // no need to transfer
    if transfer_amount == 0 {
        return Ok(());
    }

    if transfer_amount > 0 {
        // if transfer amount is more than 0 we need to transfer lamports to the account
        let transfer_accounts = Transfer {
            from: payer.to_account_info(),
            to: account.to_account_info(),
        };

        transfer(
            CpiContext::new(system_program.to_account_info(), transfer_accounts),
            transfer_amount.try_into().unwrap(),
        )
    } else {
        // if transfer amount is less than 0 this means we need to return lamports to the payer
        let transfer_to_payer_amount = transfer_amount.unsigned_abs();

        **account.try_borrow_mut_lamports()? -= transfer_to_payer_amount;
        **payer.try_borrow_mut_lamports()? += transfer_to_payer_amount;

        Ok(())
    }
}
