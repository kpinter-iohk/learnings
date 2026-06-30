# Zero-Knowledge Proofs article - handover

Status as of 2026-06-30. This file exists so the article can be resumed or extended
without re-running the research. The article itself is `/zk-proofs.qmd`.

## 1. What exists on disk

- `zk-proofs.qmd` - the article ("A Field Guide to Zero-Knowledge Proofs"). Renders clean.
- `zk-proofs.bib` - 57 references (IEEE CSL). Every inline claim that needs a source cites this.
- `zk-proofs.css` - article styles (table striping, callout accent, figure captions).
- `_quarto.yml`, `index.qmd` - navbar + index entries added.
- `cspell.config.yaml` - domain terms + bibtex-key fragments allowlisted (note: CI does NOT run
  cspell; `publish.yml` only runs `quarto render`, so cspell is local hygiene only).
- NOT committed to git yet (was on `main`; branch before committing).

## 2. Decisions that shaped the article (so you don't second-guess them)

- **Audience:** working software engineer with some CS background. Assumes hashing, public-key
  crypto, P/NP intuition, big-O. Does NOT assume finite fields / elliptic curves / abstract algebra.
- **Math depth:** intuition-first. Real machinery lives in skippable "Under the hood" callouts.
- **Validation = fact-check only.** User explicitly chose this over a backing code experiment.
  (Other articles in this repo are experiment-backed; this one is deliberately not.)
- **RISC Zero** was a required inclusion (user asked mid-session). It is the in-depth zkVM example.
- **Spaced hyphens** used throughout instead of em-dashes/unicode (global ASCII-only rule).

## 3. Article structure (10 parts + glossary)

1. Why this article exists. 2. The idea made precise (cave, 3 properties, simulator, IP=PSPACE,
proof-vs-argument, knowledge-vs-membership). 3. Schnorr/Sigma by hand + Fiat-Shamir. 4. Leap to
general computation (PCP, Kilian/Micali, arithmetization program->circuit->R1CS->QAP). 5. The
decoder ring (poly-IOP + poly-commitment + Fiat-Shamir; KZG/FRI/IPA fork). 6. Pairing family
(Pinocchio->Groth16->Sonic/PLONK/Marlin) + trusted setup. 7. Transparent family (STARK/FRI,
Bulletproofs, Halo, Nova) + comparison viz/table. 8. The *-ARK decoder table + imposters. 9. zkVMs
(RISC Zero in depth; SP1, Jolt, Cairo). 10. Applications (Zcash, Monero, rollups, zkEVM types,
long tail). 11. Cost reality. + Decision guide + glossary.

Visualizations: 5 Mermaid (Sigma sequence; arithmetization; the recipe; pairing lineage; RISC Zero
pipeline) + 1 OJS/d3 (proof-size log-scale bar chart). Two `WIDGET_IDEA` HTML comments mark future
interactive widgets (Schnorr playground; arithmetization visualizer).

## 4. Verified research digest (the expensive part to reproduce)

Cross-checked against primary sources. "[FLAG]" = a caveat the researchers explicitly raised.
Many of these facts are NOT in the article yet - they are raw material for expansion.

### Foundations / history
- GMR introduced ZK: STOC 1985 conf, SIAM J. Comput. 1989 journal. First concrete ZK proof was
  quadratic residuosity, NOT graphs.
- Graph isomorphism + 3-coloring ZK and "all NP has ZK": GMW, FOCS 1986 / JACM 1991. [FLAG: GMW != GMR.]
- Ali Baba cave: Quisquater, Guillou, Berson et al., CRYPTO '89 (LNCS 435, 1990), pp.628-631.
- Arthur-Merlin (public coin): Babai STOC 1985. Private vs public coin equivalent: Goldwasser-Sipser
  STOC 1986. IP=PSPACE: Shamir FOCS 1990 / JACM 1992, built on LFKN arithmetization.
- Schnorr: CRYPTO '89 ("...Identification and Signatures for Smart Cards") vs J.Cryptology 1991
  ("Efficient Signature Generation...") - [FLAG: titles differ, don't conflate]. Patent US 4,995,082,
  expired Feb 2008.
- Special soundness extractor for Schnorr: x = (z - z')/(c - c'). HVZK sim: pick z,c, set a = g^z y^-c.
- Fiat-Shamir: CRYPTO '86 (LNCS 263), pp.186-194 [FLAG: a secondary source mislabels it Eurocrypt '86 - wrong].
  ROM introduced by Bellare-Rogaway CCS 1993. FS standard-model insecurity: Goldwasser-Tauman Kalai FOCS 2003.
- Proof vs argument (computational soundness): Brassard-Chaum-Crepeau JCSS 1988. Knowledge extractor
  formalized: Bellare-Goldreich CRYPTO '92.
- PCP theorem (NP=PCP[O(log n),O(1)]): Arora-Safra + ALMSS, both FOCS 1992 / JACM 1998. Godel Prize 2001.
- Kilian STOC 1992 (succinct args from PCP+Merkle). Micali "CS Proofs" FOCS 1994 / SIAM 2000 (non-interactive via FS).
- "SNARK" coined: Bitansky-Canetti-Chiesa-Tromer, ITCS 2012 [FLAG: "coined here" is secondary attribution;
  paper body was 403 to the fetcher].
- "Succinct" = polylogarithmic (strict, per Thaler) vs sublinear (loose). No single fixed threshold. [FLAG]
- Repetition: sequential preserves ZK (more rounds); parallel can BREAK ZK (Goldreich-Krawczyk).

### SNARK zoo (pairing / KZG)
- Pipeline program->circuit->R1CS->QAP: Vitalik "QAP from Zero to Hero," Dec 2016. Divisibility check:
  t(x) | A(x)B(x)-C(x), t(x)=(x-1)...(x-n).
- Pinocchio: Parno-Howell-Gentry-Raykova, IEEE S&P 2013, 288-byte proofs, ~10ms verify. [FLAG: title is
  "Nearly Practical"; builds on GGPR 2012/215, not the originator. Author order differs across sources.]
- Groth16: Groth, EUROCRYPT 2016, eprint 2016/260. 3 group elements (2 G1 + 1 G2), ~128-200B
  [FLAG: byte size is curve/serialization dependent]. Verify = one pairing-product equation = ~3-4
  pairings [FLAG: NOT "one pairing"]. Per-circuit trusted setup.
- KZG poly commitments: Kate-Zaverucha-Goldberg, ASIACRYPT 2010. No eprint number (CACR 2010-10).
- Sonic (CCS 2019, 2019/099) = first practical universal+updatable SRS. [FLAG: the updatable-SRS
  concept predates it - Groth-Kohlweiss-Maller-Meiklejohn-Miers, CRYPTO 2018, 2018/280.]
- PLONK: Gabizon-Williamson-Ciobotaru, 2019/953. Acronym = "Permutations over Lagrange-bases for
  Oecumenical Noninteractive arguments of Knowledge." Custom gates + permutation/copy-constraint argument.
- Marlin: Chiesa et al., 2019/1047, EUROCRYPT 2020. Universal+updatable for R1CS (holography).
- HyperPlonk: Chen-Bunz-Boneh-Zhang, EUROCRYPT 2023, 2022/1355. Hypercube + multilinear PCS, no FFTs.
- plookup: Gabizon-Williamson 2020/315. Lookup tables; pairs with Plonkish.
- Curves: BN254/alt_bn128 (EIP-196/197/1108) dropped to ~100-110 bit security after exTNFS
  (Kim-Barbulescu CRYPTO 2016). BLS12-381 = Bowe/Zcash 2017 response, ~128-bit target.
- arkworks (Rust), gnark (Go), circom (DSL->R1CS), snarkjs (JS) are TOOLS, not proof systems.

### Transparent / hash-based / folding
- STARK: Ben-Sasson-Bentov-Horesh-Riabzev, eprint 2018/046. = Scalable Transparent ARgument of Knowledge.
  AIR (execution trace + adjacent-row constraints). Proofs "few hundred KB" per Vitalik [FLAG: tens-to-hundreds
  KB is more typical; "hundreds" is the high end].
- FRI: same authors, ICALP 2018 / ECCC 2017/134. [FLAG: one agent mis-cited FRI as eprint 2017/1066 -
  that number is Bulletproofs. Corrected.] Prover <6N, verifier <=21 log N.
- Bulletproofs: Bunz-Bootle-Boneh-Poelstra-Wuille-Maxwell, eprint 2017/1066, S&P 2018. Log proofs,
  no setup, discrete-log, LINEAR verify. IPA core (from Bootle et al. EUROCRYPT 2016). Monero RingCT
  range proofs (activated Oct 2018; Bulletproofs+ Aug 2022). Monero is NOT a SNARK system.
- Halo: Bowe-Grigg-Hopwood, eprint 2019/1021. Recursion w/o trusted setup via accumulation, IPA.
  Halo 2 = LATE 2020 (PLONKish + swappable backend) [FLAG: the article originally wrongly said Halo2=2019; fixed].
  Zcash Orchard on Halo2 since NU5, mainnet 31 May 2022, no trusted setup.
- Nova (folding, relaxed R1CS): Kothapalli-Setty-Tzialla, eprint 2021/370, CRYPTO 2022. ~2 group
  scalar-mults/step. SuperNova (2022/1758, non-uniform IVC), HyperNova (2023/573, CCS via sum-check),
  ProtoStar (2023/620, high-degree PLONK gates + lookups).
- Spartan: Setty, eprint 2019/550, CRYPTO 2020. Transparent R1CS via sum-check, multilinear PCS, no FFTs.
- Ligero (CCS 2017, sqrt(N)), Brakedown (2021/1043, CRYPTO 2023, linear-time, field-agnostic). Code-based,
  plausibly post-quantum.
- Sum-check: Lund-Fortnow-Karloff-Nisan, FOCS 1990 / JACM 1992 [FLAG: cite 1990, not 1992]. Engine of
  GKR, Spartan, HyperPlonk, HyperNova, Lasso/Jolt.
- PCS taxonomy: KZG (pairing, setup, constant, not PQ) / FRI (hash, transparent, polylog, PQ) /
  IPA (discrete-log, transparent, log size, linear verify, not PQ) / code-based (hash, transparent, PQ, larger).

### zkVMs + applications + 2026 landscape
- zkVM concept: build the CPU circuit once, prove any compiled binary. Replaces circuit DSLs.
- RISC Zero: RV32IM RISC-V, ECALL for crypto. Terms: guest, host, image ID (program fingerprint),
  journal (public output), seal (attestation), receipt (= journal + seal). Pipeline: executor -> trace ->
  segments (continuations, since v0.15) -> per-segment STARK via FRI (DEEP-ALI + batched FRI) ->
  recursion into one STARK -> Groth16 wrap (needs one-time trusted setup, ceremony 2024 -
  [FLAG: it is a circuit-specific phase-2 on Perpetual-Powers-of-Tau phase-1; wrapper is fixed so they can
  upgrade the proving system without re-running it]) -> tiny on-chain proof. R0VM 2.0 (Apr 2025):
  ~Ethereum-block proving 35min->44s [FLAG: vendor-stated; blog 500'd]. Stable risc0-zkvm v3.0.5 (2026-02-03),
  prerelease v5.0.0-rc.1. Bonsai (hosted) -> Boundless (decentralized proof market, Base mainnet ~Sep 2025).
  [FLAG: RISC Zero formal verification is determinism-only / partial - do not overstate.]
- SP1 (Succinct): RV32IM, originally Plonky3 (STARK/FRI, BabyBear, Poseidon2). Hypercube (May 2025) ->
  multilinear sum-check (Jagged PCS, LogUp-GKR); mainnet Feb 2026, 62-opcode formal verification claim.
  [FLAG: both RISC Zero and SP1 advertise "first formally verified RISC-V zkVM" - scoped differently;
  present as competing vendor claims, do not adjudicate.]
- Jolt (a16z 2024): zkVM mostly from lookups (Lasso, 2023/1216) + sum-check; "lookup singularity"
  (attributed to Barry Whitehat). Alpha, not production.
- Cairo/StarkNet: STARK-native language+VM (not RISC-V). Valida, Nexus, OpenVM also exist.
- Zcash timeline: Sprout/BCTV14 (28 Oct 2016, setup), Sapling/Groth16 (28 Oct 2018, setup),
  Orchard/Halo2 (31 May 2022, NO setup). [FLAG: NU5 is MAY 2022 not Nov - common error.] BCTV14
  counterfeiting bug discovered 2018, disclosed Feb 2019 (CVE-2019-7167).
- ** Orchard/Halo2 soundness bug (added to article):** under-constrained variable-base scalar-mult
  gadget in halo2_gadgets, live since 2022 launch, found 29 May 2026 by Taylor Hornby via AI-assisted
  audit, patched NU6.2 hard fork 3 Jun 2026, potential undetectable counterfeiting, no evidence of
  exploitation. Sources: blocksec.com Orchard analysis; ECC blog.
- Rollups: validity (proof up front, no challenge window) vs optimistic (fraud proof, ~1wk window).
  StarkNet=STARK; zkSync Boojum=FRI/RedShift wrapped in PLONK/KZG (NOT pure STARK [FLAG]); Polygon
  zkEVM live prover=Plonky2 ([FLAG] Plonky3 is a toolkit, not a confirmed cutover); Linea=Vortex->gnark/PLONK.
  [FLAG: Scroll is NO LONGER Halo2 - switched to OpenVM in Euclid upgrade Apr 2025. Deliberately omitted
  from article to avoid the stale "Scroll=Halo2 zkEVM" claim.]
- zkEVM Type 1-4: Vitalik, 4 Aug 2022 (later informal "Type 2.5"). Trades EVM-equivalence vs prover-friendliness.
- Long tail: World ID (Semaphore), zk-email (DKIM in-circuit), zkTLS (TLSNotary/Reclaim), proof of
  reserves (Binance zk-SNARK Feb 2023), zkML (EZKL/ONNX).
- Cost: prover overhead "approaching one million times" native (a16z Mar 2025); best (Jolt) <100,000x;
  target <=1,000x. Verify cheap: Groth16 ~260B/~270k gas, PLONK ~868B/~300k gas. EF real-time proving:
  target spec Jul 2025 (P99<=10s), declared achieved Dec 2025 (~16min->16s, cost ~45x down). [FLAG: keep
  EF "45x cost" distinct from ethproofs "~169x" - different metrics.] GPU standard (ICICLE/sppark); ASIC
  efforts (Cysic, Fabric). Proof markets: Succinct Prover Network, Boundless, Lagrange.

## 5. Fact-check audit

Corrections APPLIED to the article (post-write adversarial pass, independently re-verified):
- Halo 2 dated 2020 not 2019. - Pinocchio "first practical" -> "nearly practical, built on GGPR 2012."
- Sum-check dated 1990 not 1992. - Added the 2026 Orchard soundness bug + reframed Zcash from "clean
  endpoint." - Tightened the rollup-internals sentence (zkSync/Polygon/Linea specifics).

Optional polish NOT done (judgment calls - revisit only if expanding):
- Could note updatable-SRS predates Sonic (GKMMM 2018). - Could add "Type 2.5" zkEVM. - Could state
  Groth16 verify is ~3-4 pairings (currently "single pairing equation," which is correct phrasing).
- STARK proof size "few hundred KB" is Vitalik's figure (sourced) but high end.

## 6. Open threads / expansion hooks

- Two WIDGET_IDEA markers in the qmd: interactive Schnorr playground; arithmetization visualizer.
  Repo precedent for OJS widgets: llm-failure-modes.qmd, claude-code-models.qmd.
- Only ~1/3 of the research above is used. Natural expansions: a deeper "how FRI folds" box; a
  worked R1CS->QAP example with numbers; a recursion/IVC section; a Monero-vs-Zcash privacy comparison.
- If the user ever reverses the "fact-check only" decision, the obvious experiment is a pure-Python
  Schnorr + Fiat-Shamir demo (no heavy deps) under this directory, mirroring repo convention.
- Not committed. Suggested: branch, then commit qmd/bib/css/_quarto.yml/index.qmd/cspell + this file.
