use anchor_lang::prelude::*;

declare_id!("6rqe9hyjaFbcXfg5egDV8kHsccyxGt2kTjmDjzTGqfck");

#[program]
pub mod solana_token_presale {
    use super::*;

    // Initialize the presale with the admin account
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let presale = &mut ctx.accounts.presale;
        presale.admin = *ctx.accounts.admin.key;
        presale.total_deposits = 0;
        Ok(())
    }

    // User deposits lamports (SOL) into the presale
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let presale = &mut ctx.accounts.presale;
        let user = &mut ctx.accounts.user;
    
        // Ensure user has sufficient balance
        require!(user.to_account_info().lamports() >= amount, ErrorCode::InsufficientFunds);
    
        // Transfer lamports from user to the presale account
        **user.to_account_info().try_borrow_mut_lamports()? -= amount;
        **presale.to_account_info().try_borrow_mut_lamports()? += amount;
    
        // Update user's deposit balance and total deposits
        let user_balance = &mut ctx.accounts.user_balance;
        user_balance.amount += amount;
        presale.total_deposits += amount;
    
        Ok(())
    }
    

    // Check the balance of a user
    pub fn balance_of(ctx: Context<BalanceOf>) -> Result<u64> {
        let user_balance = &ctx.accounts.user_balance;
        Ok(user_balance.amount)
    }

    // Withdraw function for admin only
    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        let presale = &mut ctx.accounts.presale;
        
        // Only admin can withdraw
        require!(presale.admin == *ctx.accounts.admin.key, ErrorCode::Unauthorized);

        let amount = presale.to_account_info().lamports();
        
        // Transfer all lamports from the presale account to the admin
        **presale.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.admin.to_account_info().try_borrow_mut_lamports()? += amount;

        // Reset total deposits after withdrawal
        presale.total_deposits = 0;
        Ok(())
    }
}

// Error Codes
#[error_code]
pub enum ErrorCode {
    #[msg("User has insufficient funds for this transaction.")]
    InsufficientFunds,
    #[msg("Only the admin can execute this transaction.")]
    Unauthorized,
}

// Account Structures
#[account]
pub struct Presale {
    pub admin: Pubkey,
    pub total_deposits: u64,
}

#[account]
pub struct UserBalance {
    pub amount: u64,
}

// Contexts
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = admin, space = 8 + 32 + 8)]
    pub presale: Account<'info, Presale>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub presale: Account<'info, Presale>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(init_if_needed, payer = user, space = 8 + 8, seeds = [b"user_balance", user.key().as_ref()], bump)]
    pub user_balance: Account<'info, UserBalance>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BalanceOf<'info> {
    #[account(mut)]
    pub user_balance: Account<'info, UserBalance>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub presale: Account<'info, Presale>,
    #[account(mut)]
    pub admin: Signer<'info>,
}
