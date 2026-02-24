// TaleNode Runtime Dialogue Runner for Unreal Engine
// UObject-based class for running TaleNode exported JSON dialogues.
// Requires: JsonUtilities module (built-in UE).
//
// Usage (Blueprint or C++):
//   UTaleNodeRunner* Runner = NewObject<UTaleNodeRunner>();
//   Runner->OnDialogueLine.AddDynamic(this, &AMyActor::HandleDialogueLine);
//   Runner->LoadDialogue(TEXT("/Game/Dialogues/intro.json"));
//   Runner->Start();

#pragma once

#include "CoreMinimal.h"
#include "UObject/NoExportTypes.h"
#include "Dom/JsonObject.h"
#include "Dom/JsonValue.h"
#include "Serialization/JsonReader.h"
#include "Serialization/JsonSerializer.h"
#include "TaleNodeRunner.generated.h"

// --- Delegates ---

DECLARE_DYNAMIC_MULTICAST_DELEGATE(FOnDialogueStarted);
DECLARE_DYNAMIC_MULTICAST_DELEGATE_OneParam(FOnDialogueEnded, const FString&, Tag);
DECLARE_DYNAMIC_MULTICAST_DELEGATE_SixParams(
    FOnDialogueLine,
    const FString&, Speaker,
    const FString&, Text,
    const FString&, Emotion,
    const FString&, Portrait,
    const FString&, Audio,
    const FString&, NodeId);
DECLARE_DYNAMIC_MULTICAST_DELEGATE_TwoParams(
    FOnChoicePresented,
    const FString&, Prompt,
    const TArray<FString>&, Options);
DECLARE_DYNAMIC_MULTICAST_DELEGATE_TwoParams(
    FOnVariableChanged,
    const FString&, Key,
    const FString&, Value);
DECLARE_DYNAMIC_MULTICAST_DELEGATE_ThreeParams(
    FOnEventTriggered,
    const FString&, Action,
    const FString&, Key,
    const FString&, Value);

// --- TaleValue ---

UENUM(BlueprintType)
enum class ETaleValueType : uint8
{
    Bool,
    Int,
    Float,
    Text
};

USTRUCT(BlueprintType)
struct FTaleValue
{
    GENERATED_BODY()

    UPROPERTY(BlueprintReadWrite)
    ETaleValueType Type = ETaleValueType::Int;

    UPROPERTY(BlueprintReadWrite)
    bool BoolVal = false;

    UPROPERTY(BlueprintReadWrite)
    int64 IntVal = 0;

    UPROPERTY(BlueprintReadWrite)
    double FloatVal = 0.0;

    UPROPERTY(BlueprintReadWrite)
    FString TextVal;

    static FTaleValue FromBool(bool V);
    static FTaleValue FromInt(int64 V);
    static FTaleValue FromFloat(double V);
    static FTaleValue FromText(const FString& V);
    static FTaleValue FromJsonValue(const TSharedPtr<FJsonValue>& V);

    bool ToBool() const;
    double ToDouble() const;
    FString ToString() const;
    bool IsNumeric() const;
};

// --- UTaleNodeRunner ---

UCLASS(BlueprintType, Blueprintable)
class UTaleNodeRunner : public UObject
{
    GENERATED_BODY()

public:
    // --- Events ---

    UPROPERTY(BlueprintAssignable)
    FOnDialogueStarted OnDialogueStarted;

    UPROPERTY(BlueprintAssignable)
    FOnDialogueEnded OnDialogueEnded;

    UPROPERTY(BlueprintAssignable)
    FOnDialogueLine OnDialogueLine;

    UPROPERTY(BlueprintAssignable)
    FOnChoicePresented OnChoicePresented;

    UPROPERTY(BlueprintAssignable)
    FOnVariableChanged OnVariableChanged;

    UPROPERTY(BlueprintAssignable)
    FOnEventTriggered OnEventTriggered;

    // --- Public API ---

    /** Load dialogue from a JSON file path. Returns true on success. */
    UFUNCTION(BlueprintCallable, Category = "TaleNode")
    bool LoadDialogue(const FString& JsonPath);

    /** Load dialogue from a JSON string. Returns true on success. */
    UFUNCTION(BlueprintCallable, Category = "TaleNode")
    bool LoadFromString(const FString& JsonString);

    /** Start the dialogue. Optionally specify a start node ID. */
    UFUNCTION(BlueprintCallable, Category = "TaleNode")
    void Start(const FString& StartNodeId = TEXT(""));

    /** Advance to the next node (after a dialogue line). */
    UFUNCTION(BlueprintCallable, Category = "TaleNode")
    void Advance();

    /** Choose an option by index (after OnChoicePresented). */
    UFUNCTION(BlueprintCallable, Category = "TaleNode")
    void Choose(int32 OptionIndex);

    /** Get a runtime variable value. */
    UFUNCTION(BlueprintCallable, Category = "TaleNode")
    FTaleValue GetVariable(const FString& Name) const;

    /** Set a runtime variable value. */
    UFUNCTION(BlueprintCallable, Category = "TaleNode")
    void SetVariable(const FString& Name, const FTaleValue& Value);

    /** Returns true if the dialogue is currently running. */
    UFUNCTION(BlueprintCallable, Category = "TaleNode")
    bool IsRunning() const { return bRunning; }

    /** Stop the dialogue immediately. */
    UFUNCTION(BlueprintCallable, Category = "TaleNode")
    void Stop();

private:
    // State
    TSharedPtr<FJsonObject> Data;
    TMap<FString, TSharedPtr<FJsonObject>> NodeMap;
    TMap<FString, FTaleValue> Variables;
    TMap<FString, TSharedPtr<FJsonObject>> Characters;
    FString CurrentNodeId;
    bool bRunning = false;
    TArray<TSharedPtr<FJsonObject>> CurrentFilteredOptions;

    // Internal helpers
    TSharedPtr<FJsonObject> GetCurrentNode() const;
    void GoTo(const FString& NextId);
    void End(const FString& Tag);
    void ProcessNode();
    void ProcessDialogue(const TSharedPtr<FJsonObject>& Node);
    void ProcessChoice(const TSharedPtr<FJsonObject>& Node);
    void ProcessCondition(const TSharedPtr<FJsonObject>& Node);
    void ProcessEvent(const TSharedPtr<FJsonObject>& Node);
    void ProcessRandom(const TSharedPtr<FJsonObject>& Node);

    bool EvaluateCondition(const TSharedPtr<FJsonObject>& Cond) const;
    bool EvaluateConditionFields(
        const FString& VarName,
        const FString& Op,
        const TSharedPtr<FJsonValue>& Value) const;
    static bool LooseEqual(const FTaleValue& A, const FTaleValue& B);

    /** Interpolate {variable} references in text. */
    static FString InterpolateText(
        const FString& Text,
        const TMap<FString, FTaleValue>& Vars);
};
