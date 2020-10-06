------------------------- MODULE Counterexample -------------------------

\* Transition 0: State 0 to State 1:

ConstInit == Proposer = 0 :> "a3" @@ 1 :> "f4" @@ 2 :> "c1"

\* Transition 0: State 0 to State 1:

State1 ==
  Proposer = 0 :> "a3" @@ 1 :> "f4" @@ 2 :> "c1"
    /\ decision = "c1" :> "None"
    /\ evidence = {}
    /\ lockedRound = "c1" :> -1
    /\ lockedValue = "c1" :> "None"
    /\ msgsPrecommit
      = 0
          :> { [id |-> "v0", round |-> 0, src |-> "a2", type |-> "PRECOMMIT"],
            [id |-> "v0", round |-> 0, src |-> "a3", type |-> "PRECOMMIT"],
            [id |-> "v0", round |-> 0, src |-> "f4", type |-> "PRECOMMIT"],
            [id |-> "v1", round |-> 0, src |-> "a2", type |-> "PRECOMMIT"],
            [id |-> "v1", round |-> 0, src |-> "a3", type |-> "PRECOMMIT"],
            [id |-> "v1", round |-> 0, src |-> "f4", type |-> "PRECOMMIT"],
            [id |-> "v2", round |-> 0, src |-> "a2", type |-> "PRECOMMIT"],
            [id |-> "v2", round |-> 0, src |-> "a3", type |-> "PRECOMMIT"],
            [id |-> "v2", round |-> 0, src |-> "f4", type |-> "PRECOMMIT"] }
        @@ 1
          :> { [id |-> "v0", round |-> 1, src |-> "a2", type |-> "PRECOMMIT"],
            [id |-> "v0", round |-> 1, src |-> "a3", type |-> "PRECOMMIT"],
            [id |-> "v0", round |-> 1, src |-> "f4", type |-> "PRECOMMIT"],
            [id |-> "v1", round |-> 1, src |-> "a2", type |-> "PRECOMMIT"],
            [id |-> "v1", round |-> 1, src |-> "a3", type |-> "PRECOMMIT"],
            [id |-> "v1", round |-> 1, src |-> "f4", type |-> "PRECOMMIT"],
            [id |-> "v2", round |-> 1, src |-> "a2", type |-> "PRECOMMIT"],
            [id |-> "v2", round |-> 1, src |-> "a3", type |-> "PRECOMMIT"],
            [id |-> "v2", round |-> 1, src |-> "f4", type |-> "PRECOMMIT"] }
        @@ 2
          :> { [id |-> "v0", round |-> 2, src |-> "a2", type |-> "PRECOMMIT"],
            [id |-> "v0", round |-> 2, src |-> "a3", type |-> "PRECOMMIT"],
            [id |-> "v0", round |-> 2, src |-> "f4", type |-> "PRECOMMIT"],
            [id |-> "v1", round |-> 2, src |-> "a2", type |-> "PRECOMMIT"],
            [id |-> "v1", round |-> 2, src |-> "a3", type |-> "PRECOMMIT"],
            [id |-> "v1", round |-> 2, src |-> "f4", type |-> "PRECOMMIT"],
            [id |-> "v2", round |-> 2, src |-> "a2", type |-> "PRECOMMIT"],
            [id |-> "v2", round |-> 2, src |-> "a3", type |-> "PRECOMMIT"],
            [id |-> "v2", round |-> 2, src |-> "f4", type |-> "PRECOMMIT"] }
    /\ msgsPrevote
      = 0
          :> { [id |-> "v0", round |-> 0, src |-> "a2", type |-> "PREVOTE"],
            [id |-> "v0", round |-> 0, src |-> "a3", type |-> "PREVOTE"],
            [id |-> "v0", round |-> 0, src |-> "f4", type |-> "PREVOTE"],
            [id |-> "v1", round |-> 0, src |-> "a2", type |-> "PREVOTE"],
            [id |-> "v1", round |-> 0, src |-> "a3", type |-> "PREVOTE"],
            [id |-> "v1", round |-> 0, src |-> "f4", type |-> "PREVOTE"],
            [id |-> "v2", round |-> 0, src |-> "a2", type |-> "PREVOTE"],
            [id |-> "v2", round |-> 0, src |-> "a3", type |-> "PREVOTE"],
            [id |-> "v2", round |-> 0, src |-> "f4", type |-> "PREVOTE"] }
        @@ 1
          :> { [id |-> "v0", round |-> 1, src |-> "a2", type |-> "PREVOTE"],
            [id |-> "v0", round |-> 1, src |-> "a3", type |-> "PREVOTE"],
            [id |-> "v0", round |-> 1, src |-> "f4", type |-> "PREVOTE"],
            [id |-> "v1", round |-> 1, src |-> "a2", type |-> "PREVOTE"],
            [id |-> "v1", round |-> 1, src |-> "a3", type |-> "PREVOTE"],
            [id |-> "v1", round |-> 1, src |-> "f4", type |-> "PREVOTE"],
            [id |-> "v2", round |-> 1, src |-> "a2", type |-> "PREVOTE"],
            [id |-> "v2", round |-> 1, src |-> "a3", type |-> "PREVOTE"],
            [id |-> "v2", round |-> 1, src |-> "f4", type |-> "PREVOTE"] }
        @@ 2
          :> { [id |-> "v0", round |-> 2, src |-> "a2", type |-> "PREVOTE"],
            [id |-> "v0", round |-> 2, src |-> "a3", type |-> "PREVOTE"],
            [id |-> "v0", round |-> 2, src |-> "f4", type |-> "PREVOTE"],
            [id |-> "v1", round |-> 2, src |-> "a2", type |-> "PREVOTE"],
            [id |-> "v1", round |-> 2, src |-> "a3", type |-> "PREVOTE"],
            [id |-> "v1", round |-> 2, src |-> "f4", type |-> "PREVOTE"],
            [id |-> "v2", round |-> 2, src |-> "a2", type |-> "PREVOTE"],
            [id |-> "v2", round |-> 2, src |-> "a3", type |-> "PREVOTE"],
            [id |-> "v2", round |-> 2, src |-> "f4", type |-> "PREVOTE"] }
    /\ msgsPropose
      = 0
          :> { [proposal |-> "v0",
              round |-> 0,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> -1],
            [proposal |-> "v0",
              round |-> 0,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> 0],
            [proposal |-> "v0",
              round |-> 0,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> 1],
            [proposal |-> "v0",
              round |-> 0,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> "v0",
              round |-> 0,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> -1],
            [proposal |-> "v0",
              round |-> 0,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> 0],
            [proposal |-> "v0",
              round |-> 0,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> 1],
            [proposal |-> "v0",
              round |-> 0,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> "v0",
              round |-> 0,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> -1],
            [proposal |-> "v0",
              round |-> 0,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 0],
            [proposal |-> "v0",
              round |-> 0,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 1],
            [proposal |-> "v0",
              round |-> 0,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> "v1",
              round |-> 0,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> -1],
            [proposal |-> "v1",
              round |-> 0,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> 0],
            [proposal |-> "v1",
              round |-> 0,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> 1],
            [proposal |-> "v1",
              round |-> 0,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> "v1",
              round |-> 0,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> -1],
            [proposal |-> "v1",
              round |-> 0,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> 0],
            [proposal |-> "v1",
              round |-> 0,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> 1],
            [proposal |-> "v1",
              round |-> 0,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> "v1",
              round |-> 0,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> -1],
            [proposal |-> "v1",
              round |-> 0,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 0],
            [proposal |-> "v1",
              round |-> 0,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 1],
            [proposal |-> "v1",
              round |-> 0,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> "v2",
              round |-> 0,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> -1],
            [proposal |-> "v2",
              round |-> 0,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> 0],
            [proposal |-> "v2",
              round |-> 0,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> 1],
            [proposal |-> "v2",
              round |-> 0,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> "v2",
              round |-> 0,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> -1],
            [proposal |-> "v2",
              round |-> 0,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> 0],
            [proposal |-> "v2",
              round |-> 0,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> 1],
            [proposal |-> "v2",
              round |-> 0,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> "v2",
              round |-> 0,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> -1],
            [proposal |-> "v2",
              round |-> 0,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 0],
            [proposal |-> "v2",
              round |-> 0,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 1],
            [proposal |-> "v2",
              round |-> 0,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 2] }
        @@ 1
          :> { [proposal |-> "v0",
              round |-> 1,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> -1],
            [proposal |-> "v0",
              round |-> 1,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> 0],
            [proposal |-> "v0",
              round |-> 1,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> 1],
            [proposal |-> "v0",
              round |-> 1,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> "v0",
              round |-> 1,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> -1],
            [proposal |-> "v0",
              round |-> 1,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> 0],
            [proposal |-> "v0",
              round |-> 1,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> 1],
            [proposal |-> "v0",
              round |-> 1,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> "v0",
              round |-> 1,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> -1],
            [proposal |-> "v0",
              round |-> 1,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 0],
            [proposal |-> "v0",
              round |-> 1,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 1],
            [proposal |-> "v0",
              round |-> 1,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> "v1",
              round |-> 1,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> -1],
            [proposal |-> "v1",
              round |-> 1,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> 0],
            [proposal |-> "v1",
              round |-> 1,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> 1],
            [proposal |-> "v1",
              round |-> 1,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> "v1",
              round |-> 1,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> -1],
            [proposal |-> "v1",
              round |-> 1,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> 0],
            [proposal |-> "v1",
              round |-> 1,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> 1],
            [proposal |-> "v1",
              round |-> 1,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> "v1",
              round |-> 1,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> -1],
            [proposal |-> "v1",
              round |-> 1,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 0],
            [proposal |-> "v1",
              round |-> 1,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 1],
            [proposal |-> "v1",
              round |-> 1,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> "v2",
              round |-> 1,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> -1],
            [proposal |-> "v2",
              round |-> 1,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> 0],
            [proposal |-> "v2",
              round |-> 1,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> 1],
            [proposal |-> "v2",
              round |-> 1,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> "v2",
              round |-> 1,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> -1],
            [proposal |-> "v2",
              round |-> 1,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> 0],
            [proposal |-> "v2",
              round |-> 1,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> 1],
            [proposal |-> "v2",
              round |-> 1,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> "v2",
              round |-> 1,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> -1],
            [proposal |-> "v2",
              round |-> 1,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 0],
            [proposal |-> "v2",
              round |-> 1,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 1],
            [proposal |-> "v2",
              round |-> 1,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 2] }
        @@ 2
          :> { [proposal |-> "v0",
              round |-> 2,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> -1],
            [proposal |-> "v0",
              round |-> 2,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> 0],
            [proposal |-> "v0",
              round |-> 2,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> 1],
            [proposal |-> "v0",
              round |-> 2,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> "v0",
              round |-> 2,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> -1],
            [proposal |-> "v0",
              round |-> 2,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> 0],
            [proposal |-> "v0",
              round |-> 2,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> 1],
            [proposal |-> "v0",
              round |-> 2,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> "v0",
              round |-> 2,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> -1],
            [proposal |-> "v0",
              round |-> 2,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 0],
            [proposal |-> "v0",
              round |-> 2,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 1],
            [proposal |-> "v0",
              round |-> 2,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> "v1",
              round |-> 2,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> -1],
            [proposal |-> "v1",
              round |-> 2,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> 0],
            [proposal |-> "v1",
              round |-> 2,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> 1],
            [proposal |-> "v1",
              round |-> 2,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> "v1",
              round |-> 2,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> -1],
            [proposal |-> "v1",
              round |-> 2,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> 0],
            [proposal |-> "v1",
              round |-> 2,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> 1],
            [proposal |-> "v1",
              round |-> 2,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> "v1",
              round |-> 2,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> -1],
            [proposal |-> "v1",
              round |-> 2,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 0],
            [proposal |-> "v1",
              round |-> 2,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 1],
            [proposal |-> "v1",
              round |-> 2,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> "v2",
              round |-> 2,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> -1],
            [proposal |-> "v2",
              round |-> 2,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> 0],
            [proposal |-> "v2",
              round |-> 2,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> 1],
            [proposal |-> "v2",
              round |-> 2,
              src |-> "a2",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> "v2",
              round |-> 2,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> -1],
            [proposal |-> "v2",
              round |-> 2,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> 0],
            [proposal |-> "v2",
              round |-> 2,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> 1],
            [proposal |-> "v2",
              round |-> 2,
              src |-> "a3",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> "v2",
              round |-> 2,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> -1],
            [proposal |-> "v2",
              round |-> 2,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 0],
            [proposal |-> "v2",
              round |-> 2,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 1],
            [proposal |-> "v2",
              round |-> 2,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 2] }
    /\ round = "c1" :> 0
    /\ step = "c1" :> "PROPOSE"
    /\ validRound = "c1" :> -1
    /\ validValue = "c1" :> "None"

(* The following formula holds true in State1 and violates the invariant *)
InvariantViolation ==
  BMC!Skolem((\E r$40 \in 0 .. 2:
    BMC!Skolem((\E v$14 \in { "v0", "v1" } \union {"None"}:
      LET t_66 ==
          {
            m$53["src"]:
              m$53 \in { m$54 \in msgsPrecommit[r$40]: m$54["id"] = v$14 }
          }
        IN
        BMC!Skolem((\E t_64 \in t_66:
          BMC!Skolem((\E t_65 \in t_66: ~(t_64 = t_65)))))
        /\ BMC!Skolem((\E rr$13 \in 0 .. 2:
          rr$13 > r$40
            /\ BMC!Skolem((\E t_60$1 \in ({ "v0", "v1" } \union {"v2"})
              \union {"None"}:
              ~(t_60$1 \in {v$14})
                /\ BMC!ConstCardinality((Cardinality({
                  m$55["src"]:
                    m$55 \in
                      { m$56 \in msgsPrevote[rr$13]: m$56["id"] = t_60$1 }
                })
                  >= 3))))))))))
================================================================================
\* Created Sat Mar 21 11:30:17 CET 2020 by Apalache
\* https://github.com/konnov/apalache
