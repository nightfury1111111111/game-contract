import dotenv from "dotenv";
dotenv.config();
import {LocalTerra, LCDClient} from "@terra-money/terra.js";
import {get_server_epoch_seconds} from "./utils.js";
import {MnemonicKey} from '@terra-money/terra.js';

// Contracts
export const MintingContractPath = "./artifacts/cw20_base.wasm"
export const ClubStakingContractPath = "../artifacts/club_staking.wasm"

export const terraTestnetClient = new LCDClient({
    URL: 'https://bombay-lcd.terra.dev',
    chainID: 'bombay-12',
});
terraTestnetClient.chainID = "bombay-12";
export const localTerraClient = new LocalTerra();
localTerraClient.chainID = "localterra";

console.log("terraTestnetClient.chainID = " + terraTestnetClient.chainID);
console.log("localTerraClient.chainID = " + localTerraClient.chainID);
export const terraClient = (process.env.TERRA_CLIENT === "localTerra") ? localTerraClient : terraTestnetClient;

// export const terraClient = new LocalTerra();

// export const terraClient = new LCDClient({
//     URL: 'https://bombay-lcd.terra.dev',
//     chainID: 'bombay-12',
// });
// terraClient.chainID = "bombay-12";
// terraClient.chainID = "localterra";

// Accounts

// export const walletTest1 = terraClient.wallets.test1;
// export const walletTest2 = terraClient.wallets.test2;
// export const walletTest3 = terraClient.wallets.test3;
// export const walletTest4 = terraClient.wallets.test4;
// export const walletTest5 = terraClient.wallets.test5;
// export const walletTest6 = terraClient.wallets.test6;
// export const walletTest7 = terraClient.wallets.test7;
// export const walletTest8 = terraClient.wallets.test8;
// export const walletTest9 = terraClient.wallets.test9;
// export const walletTest10 = terraClient.wallets.test10;
// export const mint_wallet = "terra1ttjw6nscdmkrx3zhxqx3md37phldgwhggm345k";
// export const gamifiedairdrop = "terra1m46vy0jk9wck6r9mg2n8jnxw0y4g4xgl3csh9h";
// export const privatecategory = "terra1k20rlfj3ea47zjr2sp672qqscck5k5mf3uersq";
// export const marketing = "terra1wjq02nwcv6rq4zutq9rpsyq9k08rj30rhzgvt4";
// export const advisory = "terra19rgzfvlvq0f82zyy4k7whrur8x9wnpfcj5j9g7";
// export const sameerkey = "terra12g4sj6euv68kgx40k7mxu5xlm5sfat806umek7";
const mk1 = new MnemonicKey({mnemonic: "awesome festival volume rifle diagram suffer rhythm knock unlock reveal marine transfer lumber faint walnut love hover beach amazing robust oppose moon west will",});
export const mint_wallet = terraClient.wallet(mk1);

const mk2 = new MnemonicKey({mnemonic: "kiwi habit donor choice control fruit fame hamster trip aerobic juice lens lawn popular fossil taste venture furnace october income advice window opera helmet",});
export const treasury_wallet = terraClient.wallet(mk2);
// terra1gsx5474vqlguv6fhsqcz28rszm43aj46yy6090

const mk3 = new MnemonicKey({mnemonic: "job dilemma fold hurry solar strong solar priority lawsuit pass demise senior purpose useless outdoor jaguar identify enhance dirt vehicle fun nasty dragon still",});
export const liquidity_wallet = terraClient.wallet(mk3);
// terra196jgjjwelkf7s63pzhsy9e2gky0ggpr7wcdf9f

const mk5 = new MnemonicKey({mnemonic:"element final maximum lake rain jewel never typical bunker detect gold earn fancy grace heart surge auction debris embody lazy edit worry expose soon"});
export const team_wallet = terraClient.wallet(mk5);

const mkNitin = new MnemonicKey({mnemonic:"garden celery myth discover isolate dilemma width sugar enemy grief case kingdom boring guess next huge indoor cargo crime letter useful essay gold view"});
export const nitin_wallet = terraClient.wallet(mkNitin);
// terra1aqan94tvxfc0h8ux4w96sjaqpcs5x4qds0690v

const mkAjay = new MnemonicKey({mnemonic:"purse blur pitch skirt upset master relief feel pole enroll coffee change tooth live bunker federal work dry struggle little design eyebrow hope essence"});
export const ajay_wallet = terraClient.wallet(mkAjay);
// terra1s2upge2nskedaw595qug8xrq96n2qn4vgu35cv

const mkSameer = new MnemonicKey({mnemonic:"term salon nothing matrix flower click annual bomb anxiety glide castle okay payment degree umbrella clap cancel lock broom use ritual thrive price flavor"});
export const sameer_wallet = terraClient.wallet(mkSameer);
// terra1mdypjce5j5f7qamjlj726c7hgjd3mzltj2qvcc



// export const sameerkey = "terra12g4sj6euv68kgx40k7mxu5xlm5sfat806umek7";
const mkga = new MnemonicKey({mnemonic: "guess couch drip increase gossip juice bachelor wood pilot host wire august morning advice property odor book august force oak exclude craft soda bag",});
export const gamified_airdrop_wallet = terraClient.wallet(mkga);
// export const gamifiedairdrop = "terra1m46vy0jk9wck6r9mg2n8jnxw0y4g4xgl3csh9h";

const mkwa = new MnemonicKey({mnemonic: "runway now diesel vibrant suspect light love exhibit skull right promote voyage develop broom roast soup habit snap pupil liberty man warrior stone state",});
export const whitelist_airdrop_wallet = terraClient.wallet(mkwa);
// terra1tht76ys4g5txc7kpwcgne8m0m902mxn96e3k26

const mksti = new MnemonicKey({mnemonic: "garage solar dinner lawn upset february clarify cage drip jewel inherit member omit nurse pulse forest flush cannon penalty rib ladder slush element joy",});
export const star_terra_ido_wallet = terraClient.wallet(mksti);
// terra1nrsk3mdl5f6cct7v4r3ljlrfy78ay4d286autf

const mklpi = new MnemonicKey({mnemonic: "kiwi bunker found artist script slim trade away sport manage manual receive obscure leader defense void bench mobile cricket naive surge pipe dream attend",});
export const bonded_lp_reward_wallet = terraClient.wallet(mklpi);
// terra1xqqf9ktwhkpdfey9quwchpkhc4u4vesqaz772u

const mkac = new MnemonicKey({mnemonic: "code tenant find country possible pulp cream away poet flee ugly galaxy brick mean label armor fee auction guess utility luxury clump exile occur",});
export const angel_category_wallet = terraClient.wallet(mkac);
// terra12zydkmdjnv2d4ky5gk45xv6lu3a8ewa6mj8x3g

const mksc = new MnemonicKey({mnemonic: "humor shoulder differ flame aisle ski noodle undo ghost solution calm crowd finish diesel correct mountain vote dirt hollow frost apple chronic opera soft",});
export const seed_category_wallet = terraClient.wallet(mksc);
// terra1zhx5xq9ruhuw5cp2em8lrh505eeauxhpwmpwpr

const mkpc = new MnemonicKey({mnemonic: "clean antique turtle hill confirm skirt swim leader gaze replace evoke height tent olive key argue fall stool milk seed run visit eight foil",});
export const private_category_wallet = terraClient.wallet(mkpc);
export const privatecategory = "terra1k20rlfj3ea47zjr2sp672qqscck5k5mf3uersq";

const mkpp = new MnemonicKey({mnemonic: "common rare fitness goose spatial embody average half kind party gauge fee raise depend canvas sugar click pudding wrong purpose mango tonight suit tragic",});
export const pylon_public_wallet = terraClient.wallet(mkpp);

const mktsp = new MnemonicKey({mnemonic: "size decade collect shop burger among castle jelly skill witness void stomach engine charge enroll laugh appear quality renew razor pass rescue else dry",});
export const terraswap_public_wallet = terraClient.wallet(mktsp);
// terra16cp58ne9qsgynlrp4jykc0kfpnlsz30pn69m95

const mkmarketing = new MnemonicKey({mnemonic: "bread profit three cabbage guitar butter super firm more state lonely plunge grit august grid laundry discover trade dragon hazard badge journey news say",});
export const marketing_wallet = terraClient.wallet(mkmarketing);
// export const marketing = "terra1wjq02nwcv6rq4zutq9rpsyq9k08rj30rhzgvt4";

const mkbonus = new MnemonicKey({mnemonic: "hello clutch disorder turkey want shuffle you seven across kid around sniff kiwi toddler shallow cattle library jaguar claw side credit intact bleak security",});
export const bonus_wallet = terraClient.wallet(mkbonus);
// terra1z0lh038sd42r2a4kuckere66a55zj5hnst9v5t

const mkpartnership = new MnemonicKey({mnemonic: "document valve inform type cradle prison road cherry swamp shiver vital labor vehicle wide bag oak poem airport must garden solid detail engine spread",});
export const partnership_wallet = terraClient.wallet(mkpartnership);
// terra152he3l99sg95qda7ntl00vt47ysednsf9zgvfz

const mkadvisory = new MnemonicKey({mnemonic: "limit start minor rule harsh family turtle morning salmon voyage profit smart route shiver boil weird sand soccer horn assume blood robust wrist north",});
export const advisory_wallet = terraClient.wallet(mkadvisory);
// export const advisory = "terra19rgzfvlvq0f82zyy4k7whrur8x9wnpfcj5j9g7";

const mktm = new MnemonicKey({mnemonic: "clarify hen fashion future amateur civil apart unaware entire pass arena walk vanish step uniform apple teach calm middle smart all grief action slot",});
export const team_money_wallet = terraClient.wallet(mktm);
// terra18vzucnkzl255ums33pyrnv03r5cqtrduxyst44

const mkecosystem = new MnemonicKey({mnemonic: "minute better actor exchange mom tool man suffer upgrade cargo radar dizzy alone spatial cinnamon nuclear height genuine orient blossom wing scatter middle furnace",});
export const ecosystem_wallet = terraClient.wallet(mkecosystem);
// terra1p2m4am7fyp5qkwj8g6af6lnex9ns65qw6hx24d

const mkminting = new MnemonicKey({mnemonic: "awesome festival volume rifle diagram suffer rhythm knock unlock reveal marine transfer lumber faint walnut love hover beach amazing robust oppose moon west will",});
export const minting_wallet = terraClient.wallet(mkminting);
// export const mint_wallet = "terra1ttjw6nscdmkrx3zhxqx3md37phldgwhggm345k";

const mkgasfee = new MnemonicKey({mnemonic: "crew final success notable steel harbor bicycle maze open donkey off cloth adult spread kit only increase muffin alter drink caution rare garage hazard",});
export const gasfee_wallet = terraClient.wallet(mkgasfee);
// terra12v9qg0m59t02kxwkr87ztlzzyl0qkdnw4z6m9v

const mktransaction = new MnemonicKey({mnemonic: "brand relax chest wolf announce humble awful leave reopen guess scout off never captain rookie dad jaguar wrestle security detail panda athlete fork upgrade",});
export const transaction_wallet = terraClient.wallet(mktransaction);
// terra1j9zfeguw9mfe97ldj995h2jrvhz7p3m8rl3nlk

const mkrake_return = new MnemonicKey({mnemonic: "royal steel thought shift curve beach reward radar okay butter ceiling detail bamboo asset busy knock kit oxygen jar under remove advance state silver",});
export const rake_return_wallet = terraClient.wallet(mkrake_return);
// terra1z5yp64yypq3f86l04hpuhzja7ygv50tw76m0jn


// These can be the client wallets to interact
export const walletTest1 = (process.env.TERRA_CLIENT === "localTerra") ? terraClient.wallets.test1: gamified_airdrop_wallet;
export const walletTest2 = (process.env.TERRA_CLIENT === "localTerra") ? terraClient.wallets.test2: whitelist_airdrop_wallet;
export const walletTest3 = (process.env.TERRA_CLIENT === "localTerra") ? terraClient.wallets.test3: private_category_wallet;
export const walletTest4 = (process.env.TERRA_CLIENT === "localTerra") ? terraClient.wallets.test4: marketing_wallet;
export const walletTest5 = (process.env.TERRA_CLIENT === "localTerra") ? terraClient.wallets.test5: partnership_wallet;
export const walletTest6 = (process.env.TERRA_CLIENT === "localTerra") ? terraClient.wallets.test6: advisory_wallet;
export const walletTest7 = (process.env.TERRA_CLIENT === "localTerra") ? terraClient.wallets.test7: advisory_wallet;
export const walletTest10 = (process.env.TERRA_CLIENT === "localTerra") ? terraClient.wallets.test10: gasfee_wallet;


export const deployer = mint_wallet; // used as operator on all contracts
// used as operator on all contracts
// These can be the client wallets to interact


export const swapinitMessage = {
    pair_code_id: 321,
    token_code_id: 123
}

export const mintInitMessage = {
    name: "Fury",
    symbol: "FURY",
    decimals: 6,
    initial_balances: [
        {address: "terra1ttjw6nscdmkrx3zhxqx3md37phldgwhggm345k",amount: "410000000000000"},
        {address: "terra1m46vy0jk9wck6r9mg2n8jnxw0y4g4xgl3csh9h",amount: "0"},
        {address: "terra1k20rlfj3ea47zjr2sp672qqscck5k5mf3uersq",amount: "0"},
        {address: "terra1wjq02nwcv6rq4zutq9rpsyq9k08rj30rhzgvt4",amount: "0"},
        {address: "terra19rgzfvlvq0f82zyy4k7whrur8x9wnpfcj5j9g7",amount: "0"},
        {address: "terra12g4sj6euv68kgx40k7mxu5xlm5sfat806umek7",amount: "0"},
        {address: deployer.key.accAddress, amount: "010000000000000"},
        ],
    mint: {
        minter: "terra1ttjw6nscdmkrx3zhxqx3md37phldgwhggm345k",
        cap: "420000000000000"
    },
    marketing: {
        project: "crypto11.me",
        description: "This token in meant to be used for playing gamesin crypto11 world",
        marketing: "terra1wjq02nwcv6rq4zutq9rpsyq9k08rj30rhzgvt4"
    },
}